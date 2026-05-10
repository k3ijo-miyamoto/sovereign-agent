use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use common::{ChatMessage, ChatProvider, ChatRequest, EventStream, Role, StreamEvent, ToolCall};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    sse::parse_sse_line,
    wire::{
        ContentBlockMeta, Delta, MessagesRequest, SseEvent, WireContent, WireMessage, WireTool,
        ContentBlock,
    },
};

const BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: u32 = 8096;

pub struct AnthropicClient {
    http: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY が設定されていません")?;
        Ok(Self::new(api_key, BASE_URL))
    }

    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { http: Client::new(), api_key: api_key.into(), base_url: base_url.into() }
    }
}

#[async_trait]
impl ChatProvider for AnthropicClient {
    async fn chat_stream(&self, req: ChatRequest) -> Result<EventStream> {
        let url = format!("{}/v1/messages", self.base_url);

        let (system, messages) = split_system(&req.messages);
        let wire_messages =
            messages.iter().map(|m| to_wire_message(m)).collect::<Result<Vec<_>>>()?;
        let wire_tools: Vec<WireTool> = req.tools.iter().map(to_wire_tool).collect();

        let body = MessagesRequest {
            model: req.model,
            max_tokens: DEFAULT_MAX_TOKENS,
            system,
            messages: wire_messages,
            tools: wire_tools,
            stream: true,
        };

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Anthropic API への接続に失敗しました")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Anthropic API エラー {status}: {body}");
        }

        // SSE テキスト全体を受信してからイベントを処理する
        let text = response.text().await.context("レスポンス読み取り失敗")?;
        let events = parse_sse_body(&text)?;
        Ok(Box::pin(futures::stream::iter(events.into_iter().map(Ok))))
    }
}

/// SSE レスポンス全体を StreamEvent のリストに変換する
fn parse_sse_body(body: &str) -> Result<Vec<StreamEvent>> {
    let mut tool_buffers: HashMap<usize, (String, String, String)> = HashMap::new();
    let mut events = vec![];

    for line in body.lines() {
        let sse_event = match parse_sse_line(line) {
            Some(Ok(e)) => e,
            Some(Err(e)) => return Err(e),
            None => continue,
        };

        match sse_event {
            SseEvent::ContentBlockStart { index, content_block } => {
                if let ContentBlockMeta::ToolUse { id, name } = content_block {
                    tool_buffers.insert(index, (id, name, String::new()));
                }
            }
            SseEvent::ContentBlockDelta { index, delta } => match delta {
                Delta::TextDelta { text } => events.push(StreamEvent::Text(text)),
                Delta::InputJsonDelta { partial_json } => {
                    if let Some(buf) = tool_buffers.get_mut(&index) {
                        buf.2.push_str(&partial_json);
                    }
                }
            },
            SseEvent::ContentBlockStop { index } => {
                if let Some((id, name, json)) = tool_buffers.remove(&index) {
                    let arguments: Value = serde_json::from_str(&json)
                        .unwrap_or(Value::Object(Default::default()));
                    events.push(StreamEvent::ToolCalls(vec![ToolCall { id, name, arguments }]));
                }
            }
            SseEvent::MessageStop => events.push(StreamEvent::Done),
            _ => {}
        }
    }

    Ok(events)
}

// ── 変換ヘルパー ────────────────────────────────────────────────

fn split_system(messages: &[ChatMessage]) -> (Option<String>, Vec<&ChatMessage>) {
    let system = messages.iter().find(|m| m.role == Role::System).map(|m| m.content.clone());
    let rest: Vec<&ChatMessage> = messages.iter().filter(|m| m.role != Role::System).collect();
    (system, rest)
}

fn to_wire_message(msg: &ChatMessage) -> Result<WireMessage> {
    let role = match msg.role {
        Role::User | Role::Tool => "user".to_string(),
        Role::Assistant => "assistant".to_string(),
        Role::System => bail!("system メッセージは wire に変換できません"),
    };
    let content = if msg.role == Role::Tool {
        WireContent::Blocks(vec![ContentBlock::ToolResult {
            tool_use_id: msg.tool_call_id.clone().unwrap_or_default(),
            content: msg.content.clone(),
        }])
    } else {
        WireContent::Text(msg.content.clone())
    };
    Ok(WireMessage { role, content })
}

fn to_wire_tool(t: &common::ToolDef) -> WireTool {
    WireTool {
        name: t.function.name.clone(),
        description: t.function.description.clone(),
        input_schema: t.function.parameters.clone(),
    }
}
