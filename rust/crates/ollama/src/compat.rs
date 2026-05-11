/// OpenAI 互換エンドポイント (/v1/chat/completions) を使う非ストリーミングクライアント。
/// Ollama の /api/chat ストリーミングとの相性が悪いモデル（codestral 等）向け。
/// ツール API は使わず、XML モードで動作する。
use anyhow::{Context, Result};
use async_trait::async_trait;
use common::{ChatProvider, ChatRequest, EventStream, StreamEvent};
use futures::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// /v1/chat/completions を使うモデルのプレフィックス一覧
pub const COMPAT_MODE_PREFIXES: &[&str] = &["codestral", "mistral-nemo", "deepseek"];

pub struct OllamaCompatClient {
    http: Client,
    base_url: String,
}

impl OllamaCompatClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { http: Client::new(), base_url: base_url.into() }
    }

    pub fn local() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }

    pub fn is_compat_model(model: &str) -> bool {
        COMPAT_MODE_PREFIXES.iter().any(|p| model.starts_with(p))
    }
}

#[async_trait]
impl ChatProvider for OllamaCompatClient {
    fn xml_mode(&self) -> bool {
        true // ツール API を使わない。常に XML モード
    }

    async fn chat_stream(&self, req: ChatRequest) -> Result<EventStream> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let messages: Vec<WireMessage> = req.messages.iter().map(|m| {
            let role = match m.role {
                common::Role::System    => "system",
                common::Role::User      => "user",
                common::Role::Assistant => "assistant",
                common::Role::Tool      => "user", // tool ロールは user として送る
            }
            .to_string();
            WireMessage { role, content: m.content.clone() }
        }).collect();

        let body = OAIRequest { model: req.model, messages, stream: false };

        let resp: OAIResponse = self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("OpenAI compat エンドポイントへの接続に失敗")?
            .json()
            .await
            .context("OpenAI compat レスポンスのパース失敗")?;

        let content = resp.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();


        let events: Vec<Result<StreamEvent>> = vec![
            Ok(StreamEvent::Text(content)),
            Ok(StreamEvent::Done),
        ];
        Ok(Box::pin(stream::iter(events)))
    }
}

#[derive(Serialize)]
struct OAIRequest {
    model: String,
    messages: Vec<WireMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct WireMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OAIResponse {
    choices: Vec<OAIChoice>,
}

#[derive(Deserialize)]
struct OAIChoice {
    message: OAIMessageContent,
}

#[derive(Deserialize)]
struct OAIMessageContent {
    content: String,
}
