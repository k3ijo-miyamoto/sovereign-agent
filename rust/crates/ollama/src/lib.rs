mod client;
mod wire;   // Ollama API の wire format（内部型）

pub use client::OllamaClient;
// common の型を再エクスポート（利用側の利便性のため）
pub use common::{ChatMessage, ChatProvider, ChatRequest, StreamEvent, ToolDef};
