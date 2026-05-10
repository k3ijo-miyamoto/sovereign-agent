/// Ollama /api/chat のレスポンス wire format（内部用）
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: MessageDelta,
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct MessageDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<WireToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct WireToolCall {
    pub function: WireFunction,
}

#[derive(Debug, Deserialize)]
pub struct WireFunction {
    pub name: String,
    pub arguments: Value,
}
