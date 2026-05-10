use anyhow::Result;
use common::{ChatMessage, ChatProvider, ChatRequest, StreamEvent};
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
                if xml_calls.is_empty() {
                    on_text(&text_buf);
                } else {
                    let display = strip_tool_calls(&text_buf);
                    if !display.trim().is_empty() {
                        on_text(&display);
                    }
                    for (name, arguments) in xml_calls {
                        call_counter += 1;
                        pending.push((format!("call_{call_counter}"), name, arguments));
                    }
                }
            }

            if pending.is_empty() {
                if !text_buf.is_empty() {
                    history.push(ChatMessage::assistant(&text_buf));
                }
                return Ok(TurnOutcome { text: text_buf, tool_calls_executed: total_tool_calls });
            }

            history.push(ChatMessage::assistant(&text_buf));

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

fn build_xml_tool_prompt(tools: &[common::ToolDef]) -> String {
    let json = serde_json::to_string_pretty(tools).unwrap_or_default();
    format!(
        r#"

## Available tools
{json}

To call a tool output ONLY:
<tool_call>{{"name": "tool_name", "arguments": {{...}}}}</tool_call>

After receiving a tool result you MUST write a text response summarizing it."#
    )
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
            if let Ok(v) = serde_json::from_str::<Value>(inner) {
                if let Some(name) = v["name"].as_str() {
                    results.push((name.to_string(), v["arguments"].clone()));
                }
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
                if let Ok(v) = serde_json::from_str::<Value>(inner) {
                    if let Some(name) = v["name"].as_str() {
                        results.push((name.to_string(), v["arguments"].clone()));
                    }
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
