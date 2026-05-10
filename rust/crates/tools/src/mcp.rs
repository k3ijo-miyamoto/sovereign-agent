/// MCP (Model Context Protocol) クライアント
///
/// JSON-RPC 2.0 over stdio でサーバープロセスと通信する。
/// 仕様: modelcontextprotocol.io / JSON-RPC 2.0 (RFC相当)
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

// ── 設定 ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    /// .sovereign/mcp.json の "servers" キー
    /// Claude Desktop 互換として "mcpServers" も受け付ける
    #[serde(alias = "mcpServers", default)]
    pub servers: HashMap<String, McpServerConfig>,
}

// ── 発見されたツール ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub server_name: String,
}

// ── MCP サーバークライアント ─────────────────────────────────────────────

struct Inner {
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
    _child: Child,
}

pub struct McpServer {
    pub name: String,
    pub tools: Vec<McpToolDef>,
    inner: Arc<Mutex<Inner>>,
}

impl McpServer {
    pub async fn connect(name: String, cfg: &McpServerConfig) -> Result<Self> {
        let env = expand_env(&cfg.env);
        let mut child = tokio::process::Command::new(&cfg.command)
            .args(&cfg.args)
            .envs(&env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut inner = Inner { stdin, stdout, next_id: 1, _child: child };

        // MCP 初期化ハンドシェイク (spec: 2024-11-05)
        inner.request(json!({
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "clientInfo": { "name": "sovereign", "version": "0.1.0" },
                "capabilities": {}
            }
        })).await?;
        inner.notify(json!({ "method": "notifications/initialized" })).await?;

        // ツール一覧を取得
        let list = inner.request(json!({ "method": "tools/list" })).await?;
        let tools = list["result"]["tools"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|t| McpToolDef {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
                server_name: name.clone(),
            })
            .collect();

        Ok(Self { name, tools, inner: Arc::new(Mutex::new(inner)) })
    }

    pub async fn call(&self, tool: &str, args: &Value) -> Result<String> {
        let mut inner = self.inner.lock().await;
        let resp = inner.request(json!({
            "method": "tools/call",
            "params": { "name": tool, "arguments": args }
        })).await?;

        if let Some(err) = resp.get("error") {
            bail!("MCP error from {}: {err}", self.name);
        }

        // content は [{type, text}] の配列
        let text = resp["result"]["content"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_else(|| resp["result"].to_string());

        Ok(text)
    }
}

impl Inner {
    async fn request(&mut self, mut body: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        body["jsonrpc"] = json!("2.0");
        body["id"] = json!(id);
        self.write(&body).await?;

        // サーバーからの通知（id なし）を読み飛ばし、目的の応答を返す
        loop {
            let line = self.read_line().await?;
            if line.get("id") == Some(&json!(id)) {
                return Ok(line);
            }
        }
    }

    async fn notify(&mut self, mut body: Value) -> Result<()> {
        body["jsonrpc"] = json!("2.0");
        self.write(&body).await
    }

    async fn write(&mut self, v: &Value) -> Result<()> {
        let s = serde_json::to_string(v)? + "\n";
        self.stdin.write_all(s.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_line(&mut self) -> Result<Value> {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line).await?;
            let t = line.trim();
            if !t.is_empty() {
                return Ok(serde_json::from_str(t)?);
            }
        }
    }
}

// ── 設定ロード ──────────────────────────────────────────────────────────

/// .sovereign/mcp.json → ~/.config/sovereign/mcp.json の順で探す
pub fn load_config() -> Option<McpConfig> {
    let candidates = [
        std::path::PathBuf::from(".sovereign/mcp.json"),
        config_dir().join("sovereign/mcp.json"),
    ];
    for p in &candidates {
        if let Ok(s) = std::fs::read_to_string(p) {
            if let Ok(cfg) = serde_json::from_str(&s) {
                return Some(cfg);
            }
        }
    }
    None
}

fn config_dir() -> std::path::PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| std::path::PathBuf::from(".config"))
        })
}

fn expand_env(env: &HashMap<String, String>) -> HashMap<String, String> {
    env.iter()
        .map(|(k, v)| {
            let v = if v.starts_with("${") && v.ends_with('}') {
                std::env::var(&v[2..v.len() - 1]).unwrap_or_default()
            } else {
                v.clone()
            };
            (k.clone(), v)
        })
        .collect()
}
