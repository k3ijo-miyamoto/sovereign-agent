mod args;
mod plain;

use agent::{AgentLoop, History};
use anyhow::Result;
use common::ChatMessage;
use tools::{all_tool_defs, LocalExecutor};

use args::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let provider: Box<dyn common::ChatProvider> = match cli.provider.as_str() {
        "anthropic" => Box::new(anthropic::AnthropicClient::from_env()?),
        _ => Box::new(ollama::OllamaClient::new(&cli.base_url, &cli.model)),
    };

    let xml = provider.xml_mode();
    let tool_defs = all_tool_defs();
    let system = "\
You are an autonomous coding assistant with access to tools. \
You MUST use tools to complete tasks — never ask the user to provide file contents or run commands for you. \
When asked to modify a file: (1) use read_file to read it, (2) use write_file to write the updated version. \
When asked about a directory: use list_files. \
When you need to run code or shell commands: use bash. \
Always act autonomously and complete the full task with tools before responding."
        .to_string();
    let mut history = History::new(system);
    let agent = AgentLoop::new(provider, &cli.model, tool_defs);

    if cli.plain_output {
        // VS Code 拡張 / eval ハーネス向け JSON Lines モード
        if let Some(prompt) = cli.prompt {
            // 単発実行: prompt サブコマンド
            history.push(ChatMessage::user(&prompt));
            let executor = plain::PlainExecutor;
            plain::run_once(&agent, &mut history, &executor).await?;
        } else {
            // 対話 plain モード (stdin から読む)
            plain::run(agent, history, plain::PlainExecutor).await?;
        }
    } else {
        // 通常の対話 REPL
        repl::run(agent, history, LocalExecutor, &cli.model, &cli.provider, xml).await?;
    }

    Ok(())
}

mod repl {
    use agent::{AgentLoop, History};
    use anyhow::Result;
    use common::ChatMessage;
    use tools::LocalExecutor;

    pub async fn run(
        agent: AgentLoop,
        mut history: History,
        executor: LocalExecutor,
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
            let outcome = agent.run_turn(&mut history, &executor, &mut on_text).await?;
            eprintln!();
            if outcome.tool_calls_executed > 0 {
                eprintln!("[{} tool call(s) executed]", outcome.tool_calls_executed);
            }
        }
        Ok(())
    }
}
