use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::{ChatMessage, ToolCall, ToolDef};

/// エージェントループがプロバイダーに渡すリクエスト
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDef>,
}

/// ストリームの1イベント（プロバイダー共通）
#[derive(Debug)]
pub enum StreamEvent {
    Text(String),
    ToolCalls(Vec<ToolCall>),
    Done,
}

pub type EventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

/// Ollama / Anthropic / その他プロバイダーが実装するトレイト
#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat_stream(&self, req: ChatRequest) -> Result<EventStream>;

    /// XML モード（ツール定義をシステムプロンプトに埋め込むモデル）かどうか
    fn xml_mode(&self) -> bool {
        false
    }
}
