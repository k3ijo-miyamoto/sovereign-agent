mod args;
mod plain;

use agent::{AgentLoop, History};
use anyhow::Result;
use common::ChatMessage;
use std::sync::Arc;
use tools::{all_tool_defs, CombinedExecutor, load_mcp_config, McpServer};

use args::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // --cwd でカレントディレクトリを変更
    if let Some(ref cwd) = cli.cwd {
        std::env::set_current_dir(cwd)?;
    }

    let provider: Box<dyn common::ChatProvider> = match cli.provider.as_str() {
        "anthropic" => Box::new(anthropic::AnthropicClient::from_env()?),
        _ => {
            // codestral / mistral-nemo 等は /v1/chat/completions (非ストリーミング) を使う
            if ollama::OllamaCompatClient::is_compat_model(&cli.model) {
                Box::new(ollama::OllamaCompatClient::new(&cli.base_url))
            } else {
                Box::new(ollama::OllamaClient::new(&cli.base_url, &cli.model))
            }
        }
    };

    let xml = provider.xml_mode();

    // --allowed-tools でローカルツールをフィルタリング
    let mut tool_defs = if let Some(ref allowed) = cli.allowed_tools {
        let names: std::collections::HashSet<&str> =
            allowed.split(',').map(|s| s.trim()).collect();
        all_tool_defs()
            .into_iter()
            .filter(|t| names.contains(t.function.name.as_str()))
            .collect()
    } else {
        all_tool_defs()
    };

    // MCP サーバーを起動してツール一覧を取得
    let mcp_servers = init_mcp_servers().await;
    for server in &mcp_servers {
        for tool in &server.tools {
            tool_defs.push(common::ToolDef {
                kind: "function".into(),
                function: common::ToolSpec {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            });
        }
    }

    let executor: Arc<dyn agent::ToolExecutor> =
        Arc::new(CombinedExecutor::new(mcp_servers));

    let system = "\
You are an autonomous coding assistant with access to tools.\n\
\n\
Rules — follow all of them without exception:\n\
1. Always use tools to act. Never ask the user to run commands or provide file contents.\n\
2. To read a file: use read_file. To modify a file: read_file first, then write_file with the full updated content.\n\
3. To run code or shell commands: use bash.\n\
4. When code produces wrong or unexpected output, investigate the cause and fix it — never accept incorrect output.\n\
5. Keep calling tools until the task is fully and verifiably complete. Never stop mid-task.\n\
6. If an approach fails, try a different one. Do not give up.\n\
7. Reply in the same language the user used."
        .to_string();
    let mut history = History::new(system);
    let agent = AgentLoop::new(provider, &cli.model, tool_defs);

    // ビジョン設定（vision_model が指定されている場合のみ）
    let vision = cli.vision_model.as_ref().map(|m| plain::VisionCfg {
        base_url: cli.base_url.clone(),
        model: m.clone(),
    });

    if cli.plain_output {
        if let Some(prompt) = cli.prompt {
            history.push(ChatMessage::user(&prompt));
            plain::run_once(&agent, &mut history, executor.as_ref()).await?;
        } else {
            plain::run(agent, history, executor, vision).await?;
        }
    } else {
        repl::run(agent, history, executor, &cli.model, &cli.provider, xml).await?;
    }

    Ok(())
}

mod repl {
    use agent::{AgentLoop, History};
    use anyhow::Result;
    use common::ChatMessage;
    use std::sync::Arc;

    pub async fn run(
        agent: AgentLoop,
        mut history: History,
        executor: Arc<dyn agent::ToolExecutor>,
        model: &str,
        provider: &str,
        xml: bool,
    ) -> Result<()> {
        eprintln!("sovereign-agent  provider={provider}  model={model}  xml_mode={xml}");
        eprintln!("終了: Ctrl-D または /exit");

        let stdin = std::io::stdin();
        loop {
            eprint!("> ");
            let mut line = String::new();
            if stdin.read_line(&mut line)? == 0 { break; }
            let input = line.trim();
            if input.is_empty() { continue; }
            if input == "/exit" { break; }

            history.push(ChatMessage::user(input));
            let mut on_text = |chunk: &str| { eprint!("{chunk}"); };
            let outcome = agent.run_turn(&mut history, executor.as_ref(), &mut on_text).await?;
            eprintln!();
            if outcome.tool_calls_executed > 0 {
                eprintln!("[{} tool call(s) executed]", outcome.tool_calls_executed);
            }
        }
        Ok(())
    }
}

async fn init_mcp_servers() -> Vec<Arc<McpServer>> {
    let Some(cfg) = load_mcp_config() else { return vec![]; };
    let mut servers = Vec::new();
    for (name, server_cfg) in cfg.servers {
        match McpServer::connect(name.clone(), &server_cfg).await {
            Ok(server) => {
                eprintln!("[MCP] {} に接続 ({} ツール)", name, server.tools.len());
                servers.push(Arc::new(server));
            }
            Err(e) => eprintln!("[MCP] {} 接続失敗: {e}", name),
        }
    }
    servers
}
