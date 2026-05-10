/// VS Code 拡張との JSON Lines プロトコル
///
/// stdin から1行ずつユーザーメッセージを受け取り、
/// stdout に JSON Lines で応答を返す。
///
/// 通常メッセージ:   テキスト1行
/// 画像メッセージ:   {"type":"image_prompt","text":"...","base64":"...","mime":"image/png"}
///
/// stdout フォーマット:
///   {"type":"text","delta":"..."}
///   {"type":"tool_start","name":"..."}
///   {"type":"tool_done","name":"...","ok":true}
///   {"type":"ready"}
///   {"type":"error","message":"..."}

use agent::{AgentLoop, History};
use anyhow::Result;
use common::ChatMessage;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::io::Write;
use std::sync::Arc;

pub struct VisionCfg {
    pub base_url: String,
    pub model: String,
}

pub async fn run(
    agent: AgentLoop,
    mut history: History,
    executor: Arc<dyn agent::ToolExecutor>,
    vision: Option<VisionCfg>,
) -> Result<()> {
    emit("ready");

    let stdin = std::io::stdin();
    let http = Client::new();

    loop {
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "/exit" {
            break;
        }

        // image_prompt JSON を検出
        if input.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<Value>(input) {
                if val.get("type").and_then(|t| t.as_str()) == Some("image_prompt") {
                    handle_image_prompt(&val, &vision, &http).await;
                    emit("ready");
                    continue;
                }
            }
        }

        history.push(ChatMessage::user(input));

        let mut on_text = |chunk: &str| {
            let msg = json!({"type":"text","delta": chunk});
            println!("{msg}");
            let _ = std::io::stdout().flush();
        };

        let plain_exec = PlainExecutor(Arc::clone(&executor));
        match agent.run_turn(&mut history, &plain_exec, &mut on_text).await {
            Ok(_) => emit("ready"),
            Err(e) => {
                println!("{}", json!({"type":"error","message": e.to_string()}));
                emit("ready");
            }
        }
    }
    Ok(())
}

async fn handle_image_prompt(val: &Value, vision: &Option<VisionCfg>, http: &Client) {
    let text = val["text"].as_str().unwrap_or("この画像を日本語で説明してください");
    let base64 = val["base64"].as_str().unwrap_or("");

    let cfg = match vision {
        Some(c) => c,
        None => {
            println!(
                "{}",
                json!({"type":"error","message":
                    "Vision model が未設定です。Settings の Vision model に qwen2.5vl:7b 等を入力してください。"})
            );
            let _ = std::io::stdout().flush();
            return;
        }
    };

    let body = json!({
        "model": cfg.model,
        "messages": [{"role":"user","content":text,"images":[base64]}],
        "stream": true,
    });

    let resp = match http.post(format!("{}/api/chat", cfg.base_url)).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", json!({"type":"error","message": e.to_string()}));
            let _ = std::io::stdout().flush();
            return;
        }
    };

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(_) => break,
        };
        let text_chunk = match std::str::from_utf8(&bytes) {
            Ok(s) => s,
            Err(_) => break,
        };
        for chunk_line in text_chunk.lines() {
            if chunk_line.is_empty() { continue; }
            if let Ok(parsed) = serde_json::from_str::<Value>(chunk_line) {
                if let Some(content) = parsed["message"]["content"].as_str() {
                    if !content.is_empty() {
                        println!("{}", json!({"type":"text","delta": content}));
                        let _ = std::io::stdout().flush();
                    }
                }
                if parsed["done"].as_bool().unwrap_or(false) {
                    return;
                }
            }
        }
    }
}

/// eval ハーネス向け単発実行
pub async fn run_once(
    agent: &AgentLoop,
    history: &mut History,
    executor: &dyn agent::ToolExecutor,
) -> Result<()> {
    let mut full_text = String::new();
    let mut on_text = |chunk: &str| { full_text.push_str(chunk); };
    match agent.run_turn(history, executor, &mut on_text).await {
        Ok(_) => {
            print!("{full_text}");
            let _ = std::io::stdout().flush();
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn emit(type_: &str) {
    println!("{}", json!({"type": type_}));
    let _ = std::io::stdout().flush();
}

/// tool_start / tool_done イベントを stdout に emit しつつ内部実行器に委譲する
struct PlainExecutor(Arc<dyn agent::ToolExecutor>);

#[async_trait::async_trait]
impl agent::ToolExecutor for PlainExecutor {
    async fn execute(
        &self,
        name: &str,
        call_id: &str,
        arguments: &serde_json::Value,
    ) -> agent::ToolResult {
        println!("{}", json!({"type":"tool_start","name":name,"args":arguments}));
        let _ = std::io::stdout().flush();

        let result = self.0.execute(name, call_id, arguments).await;

        println!("{}", json!({"type":"tool_done","name":name,"ok":!result.is_error}));
        let _ = std::io::stdout().flush();

        result
    }
}
