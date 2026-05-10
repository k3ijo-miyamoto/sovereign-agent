use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub call_id: String,
    pub output: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self { call_id: call_id.into(), output: output.into(), is_error: false }
    }
    pub fn err(call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self { call_id: call_id.into(), output: message.into(), is_error: true }
    }
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, name: &str, call_id: &str, arguments: &Value) -> ToolResult;
}
