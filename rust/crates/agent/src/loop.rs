use anyhow::Result;
use common::{ChatMessage, ChatProvider, ChatRequest, StreamEvent, ToolCall};
use futures::StreamExt;
use serde_json::Value;

use crate::{history::History, tool::ToolExecutor};

#[derive(Debug)]
pub struct TurnOutcome {
    pub text: String,
    pub tool_calls_executed: usize,
}

pub struct AgentLoop {
    provider: Box<dyn ChatProvider>,
    model: String,
    tools: Vec<common::ToolDef>,
    xml_system_suffix: Option<String>,
}

impl AgentLoop {
    pub fn new(
        provider: Box<dyn ChatProvider>,
        model: impl Into<String>,
        tools: Vec<common::ToolDef>,
    ) -> Self {
        let xml_system_suffix = if provider.xml_mode() {
            Some(build_xml_tool_prompt(&tools))
        } else {
            None
        };
        Self { provider, model: model.into(), tools, xml_system_suffix }
    }

    pub async fn run_turn(
        &self,
        history: &mut History,
        executor: &dyn ToolExecutor,
        on_text: &mut dyn FnMut(&str),
    ) -> Result<TurnOutcome> {
        const MAX_TOOL_CALLS: usize = 25;
        let mut total_tool_calls = 0;
        let mut call_counter = 0usize;

        loop {
            if total_tool_calls >= MAX_TOOL_CALLS {
                eprintln!("[tool call limit reached: {MAX_TOOL_CALLS}]");
                return Ok(TurnOutcome { text: String::new(), tool_calls_executed: total_tool_calls });
            }
            let messages = history.build_messages_with_suffix(self.xml_system_suffix.as_deref());
            let req = ChatRequest { model: self.model.clone(), messages, tools: self.tools.clone() };
            let mut stream = self.provider.chat_stream(req).await?;

            let mut text_buf = String::new();
            let mut pending: Vec<(String, String, Value)> = vec![];

            while let Some(event) = stream.next().await {
                match event? {
                    StreamEvent::Text(chunk) => {
                        if self.provider.xml_mode() {
                            text_buf.push_str(&chunk); // バッファに溜めてあとでスキャン
                        } else {
                            on_text(&chunk);
                            text_buf.push_str(&chunk);
                        }
                    }
                    StreamEvent::ToolCalls(calls) => {
                        for c in calls {
                            pending.push((c.id, c.name, c.arguments));
                        }
                    }
                    StreamEvent::Done => break,
                }
            }

            // XML モード: バッファ完了後にスキャン
            if self.provider.xml_mode() {
                let xml_calls = extract_xml_tool_calls(&text_buf);
                if !xml_calls.is_empty() {
                    let display = strip_tool_calls(&text_buf);
                    if !display.trim().is_empty() {
                        on_text(&display);
                    }
                    for (name, arguments) in xml_calls {
                        call_counter += 1;
                        pending.push((format!("call_{call_counter}"), name, arguments));
                    }
                } else if let Some((name, arguments)) = parse_json_tool_call(&text_buf) {
                    // フォールバック: <tool_call> でも ```tool_call でもなく
                    // ```python / 生テキスト内に {"name":...,"arguments":...} を出すモデル対応
                    call_counter += 1;
                    pending.push((format!("call_{call_counter}"), name, arguments));
                } else {
                    on_text(&text_buf);
                }
            } else if pending.is_empty() {
                // native tools API フォールバック:
                // モデルが tool_calls フィールドを使わず {"name":...,"arguments":...} をテキストで返す場合に対応
                if let Some((name, arguments)) = parse_json_tool_call(&text_buf) {
                    call_counter += 1;
                    pending.push((format!("call_{call_counter}"), name, arguments));
                    text_buf.clear();
                }
            }

            if pending.is_empty() {
                if !text_buf.is_empty() {
                    history.push(ChatMessage::assistant(&text_buf));
                }
                return Ok(TurnOutcome { text: text_buf, tool_calls_executed: total_tool_calls });
            }

            // native tools API: assistant メッセージに tool_calls を乗せてコンテキストを保持する
            if self.provider.xml_mode() {
                history.push(ChatMessage::assistant(&text_buf));
            } else {
                let calls = pending.iter().map(|(id, name, args)| ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    arguments: args.clone(),
                }).collect();
                history.push(ChatMessage::assistant_with_calls(&text_buf, calls));
            }

            for (id, name, args) in &pending {
                eprintln!("[Calling {name}]");
                let result = executor.execute(name, id, &args).await;
                eprintln!("[Result: {}]", if result.is_error { "error" } else { "ok" });

                // XML モード: tool role を解さないためユーザーメッセージとして送る
                if self.provider.xml_mode() {
                    history.push(ChatMessage::user(format!(
                        "[Tool result for {name}]:\n{}",
                        result.output
                    )));
                } else {
                    history.push(ChatMessage::tool_result(&result.call_id, &result.output));
                }
                total_tool_calls += 1;
            }
        }
    }
}

/// Native tools API フォールバック: テキスト内の `{"name":...,"arguments":...}` を探して tool call として解釈する。
/// qwen2.5-coder 等 Ollama の tool_calls フィールドを使わずテキスト出力するモデルに対応。
/// prose が混じっていても JSON オブジェクトを抽出できる。
fn parse_json_tool_call(text: &str) -> Option<(String, Value)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = find_json_object_end(bytes, i) {
                let candidate = &text[i..=end];
                let sanitized = sanitize_json_string(candidate);
                if let Ok(v) = serde_json::from_str::<Value>(&sanitized) {
                    if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
                        let arguments = v.get("arguments")
                            .cloned()
                            .unwrap_or(Value::Object(Default::default()));
                        return Some((name.to_string(), arguments));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// バイト列中の `start` から始まる JSON オブジェクトの末尾インデックスを返す。
fn find_json_object_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for i in start..bytes.len() {
        let b = bytes[i];
        if escape { escape = false; continue; }
        if b == b'\\' && in_string { escape = true; continue; }
        if b == b'"' { in_string = !in_string; continue; }
        if in_string { continue; }
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
    }
    None
}

fn build_xml_tool_prompt(tools: &[common::ToolDef]) -> String {
    let json = serde_json::to_string_pretty(tools).unwrap_or_default();
    format!(
        r#"

## Available tools
{json}

To call a tool, output ONLY a tool call block — no other text before it:
<tool_call>{{"name": "tool_name", "arguments": {{...}}}}</tool_call>

Critical rules for tool use:
- ALWAYS read the file before modifying it — never assume its contents.
- NEVER output code or file content in your text response. Use write_file to write it.
- After reading a file, immediately call write_file with the complete updated content.
- After a tool result: if the task is not yet complete, call the next tool immediately.
- Only write a final text response when the task is fully and verifiably done."#
    )
}

/// JSON 文字列内のリテラル改行・タブを JSON エスケープに変換する。
/// モデルが JSON 文字列値の中に生の改行を含む不正 JSON を出力する場合の対策。
fn sanitize_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let mut in_string = false;
    let mut escape = false;
    for ch in s.chars() {
        if escape {
            escape = false;
            out.push(ch);
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            out.push(ch);
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
            continue;
        }
        if in_string {
            match ch {
                '\n' => { out.push_str("\\n"); continue; }
                '\r' => { out.push_str("\\r"); continue; }
                '\t' => { out.push_str("\\t"); continue; }
                _ => {}
            }
        }
        out.push(ch);
    }
    out
}

fn parse_tool_call_json(raw: &str) -> Option<(String, Value)> {
    let sanitized = sanitize_json_string(raw.trim());
    let v = serde_json::from_str::<Value>(&sanitized).ok()?;
    let name = v["name"].as_str()?.to_string();
    Some((name, v["arguments"].clone()))
}

/// `<tool_call>...</tool_call>` と ` ```tool_call\n...\n``` ` の両方を抽出する
fn extract_xml_tool_calls(text: &str) -> Vec<(String, Value)> {
    let mut results = vec![];

    // XML 形式: <tool_call>{...}</tool_call>
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        rest = &rest[start + "<tool_call>".len()..];
        if let Some(end) = rest.find("</tool_call>") {
            let inner = rest[..end].trim();
            if let Some(call) = parse_tool_call_json(inner) {
                results.push(call);
            }
            rest = &rest[end + "</tool_call>".len()..];
        } else {
            break;
        }
    }

    // Markdown コードブロック形式: ```tool_call\n{...}\n```
    if results.is_empty() {
        let mut rest = text;
        while let Some(start) = rest.find("```tool_call") {
            rest = &rest[start + "```tool_call".len()..];
            // 改行を読み飛ばす
            if let Some(nl) = rest.find('\n') {
                rest = &rest[nl + 1..];
            }
            if let Some(end) = rest.find("```") {
                let inner = rest[..end].trim();
                if let Some(call) = parse_tool_call_json(inner) {
                    results.push(call);
                }
                rest = &rest[end + 3..];
            } else {
                break;
            }
        }
    }

    results
}

fn strip_tool_calls(text: &str) -> String {
    // XML 形式を除去
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        out.push_str(&rest[..start]);
        if let Some(end) = rest.find("</tool_call>") {
            rest = &rest[end + "</tool_call>".len()..];
        } else {
            break;
        }
    }
    out.push_str(rest);

    // Markdown コードブロック形式を除去
    let text2 = out.clone();
    let mut out2 = String::new();
    let mut rest2 = text2.as_str();
    while let Some(start) = rest2.find("```tool_call") {
        out2.push_str(&rest2[..start]);
        if let Some(end) = rest2.find("```\n").or_else(|| rest2.rfind("```")) {
            rest2 = &rest2[end + 3..];
            // 末尾の改行も読み飛ばす
            if rest2.starts_with('\n') { rest2 = &rest2[1..]; }
        } else {
            break;
        }
    }
    out2.push_str(rest2);
    out2
}
