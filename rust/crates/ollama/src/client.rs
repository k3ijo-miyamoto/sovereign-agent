use anyhow::{Context, Result};
use async_trait::async_trait;
use common::{ChatProvider, ChatRequest, EventStream, StreamEvent, ToolCall, ToolDef};
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::wire::{ChatResponse, WireToolCall};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// XML モードを使うモデルのプレフィックス一覧
const XML_MODE_PREFIXES: &[&str] = &["gemma3", "phi4", "codestral", "devstral", "deepseek"];

pub struct OllamaClient {
    http: Client,
    base_url: String,
    xml_mode: bool,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>, model: &str) -> Self {
        let xml_mode = XML_MODE_PREFIXES.iter().any(|p| model.starts_with(p));
        Self { http: Client::new(), base_url: base_url.into(), xml_mode }
    }

    pub fn local(model: &str) -> Self {
        Self::new(DEFAULT_BASE_URL, model)
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let resp: serde_json::Value = self.http.get(&url).send().await?.json().await?;
        let names = resp["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["name"].as_str().map(String::from))
            .collect();
        Ok(names)
    }
}

#[async_trait]
impl ChatProvider for OllamaClient {
    fn xml_mode(&self) -> bool {
        self.xml_mode
    }

    async fn chat_stream(&self, req: ChatRequest) -> Result<EventStream> {
        let url = format!("{}/api/chat", self.base_url);

        // XML モードではtools フィールドを渡さない
        let wire_tools: Option<Vec<WireToolDef>> = if self.xml_mode || req.tools.is_empty() {
            None
        } else {
            Some(req.tools.iter().map(WireToolDef::from_common).collect())
        };

        let body = OllamaRequest {
            model: req.model,
            messages: req.messages.iter().map(WireMessage::from_common).collect(),
            stream: true,
            tools: wire_tools,
        };

        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Ollama への接続に失敗しました")?;

        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.context("ストリーム読み取りエラー")?;
            let text = std::str::from_utf8(&bytes).context("UTF-8 デコードエラー")?;
            parse_chunk(text)
        });

        Ok(Box::pin(stream))
    }
}

fn parse_chunk(text: &str) -> Result<StreamEvent> {
    let resp: ChatResponse =
        serde_json::from_str(text).context("Ollama レスポンスのパース失敗")?;

    // tool_calls を done より先にチェック: Ollama は done=true のチャンクに tool_calls を乗せることがある
    if let Some(calls) = resp.message.tool_calls {
        if !calls.is_empty() {
            return Ok(StreamEvent::ToolCalls(wire_calls_to_common(calls)));
        }
    }

    if resp.done {
        return Ok(StreamEvent::Done);
    }

    Ok(StreamEvent::Text(resp.message.content.unwrap_or_default()))
}

fn wire_calls_to_common(calls: Vec<WireToolCall>) -> Vec<ToolCall> {
    calls
        .into_iter()
        .enumerate()
        .map(|(i, c)| ToolCall {
            id: format!("call_{i}"),
            name: c.function.name,
            arguments: c.function.arguments,
        })
        .collect()
}

// ── Ollama wire format の送信型 ──────────────────────────────────

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<WireMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<WireToolDef>>,
}

#[derive(Serialize)]
struct WireMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<WireOutgoingToolCall>>,
}

/// Ollama が assistant メッセージ中に期待する tool_calls の送信フォーマット
#[derive(Serialize)]
struct WireOutgoingToolCall {
    function: WireOutgoingFunction,
}

#[derive(Serialize)]
struct WireOutgoingFunction {
    name: String,
    arguments: serde_json::Value,
}

impl WireMessage {
    fn from_common(m: &common::ChatMessage) -> Self {
        let role = match m.role {
            common::Role::System    => "system",
            common::Role::User      => "user",
            common::Role::Assistant => "assistant",
            common::Role::Tool      => "tool",
        }
        .to_string();
        let tool_calls = m.tool_calls.as_ref().map(|calls| {
            calls.iter().map(|c| WireOutgoingToolCall {
                function: WireOutgoingFunction {
                    name: c.name.clone(),
                    arguments: c.arguments.clone(),
                },
            }).collect()
        });
        Self { role, content: m.content.clone(), tool_call_id: m.tool_call_id.clone(), tool_calls }
    }
}

#[derive(Serialize)]
struct WireToolDef {
    #[serde(rename = "type")]
    kind: String,
    function: WireToolSpec,
}

#[derive(Serialize)]
struct WireToolSpec {
    name: String,
    description: String,
    parameters: Value,
}

impl WireToolDef {
    fn from_common(t: &ToolDef) -> Self {
        Self {
            kind: t.kind.clone(),
            function: WireToolSpec {
                name: t.function.name.clone(),
                description: t.function.description.clone(),
                parameters: t.function.parameters.clone(),
            },
        }
    }
}
