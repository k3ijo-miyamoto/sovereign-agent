#![allow(dead_code)]
/// Anthropic API の送受信 wire format
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── 送信 ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: Option<String>,
    pub messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<WireTool>,
    pub stream: bool,
}

#[derive(Serialize)]
pub struct WireMessage {
    pub role: String,          // "user" | "assistant"
    pub content: WireContent,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum WireContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Serialize)]
pub struct WireTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

// ── SSE イベント ─────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    MessageStart { message: MessageMeta },
    ContentBlockStart { index: usize, content_block: ContentBlockMeta },
    ContentBlockDelta { index: usize, delta: Delta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: MessageDeltaData },
    MessageStop,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
pub struct MessageMeta {
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockMeta {
    Text { text: String },
    ToolUse { id: String, name: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Deserialize, Debug)]
pub struct MessageDeltaData {
    pub stop_reason: Option<String>,
}
