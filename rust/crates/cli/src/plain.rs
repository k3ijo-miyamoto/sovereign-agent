/// VS Code 拡張との JSON Lines プロトコル
///
/// stdin から1行ずつユーザーメッセージを受け取り、
/// stdout に JSON Lines で応答を返す。
///
/// stdout フォーマット:
///   {"type":"text","delta":"..."}      テキスト断片（ストリーミング）
///   {"type":"tool_start","name":"...","args":{...}}  ツール開始
///   {"type":"tool_done","name":"...","ok":true}       ツール完了
///   {"type":"ready"}                   ターン完了・次の入力待ち
///   {"type":"error","message":"..."}   エラー

use agent::{AgentLoop, History};
use anyhow::Result;
use common::ChatMessage;
use serde_json::json;
use std::io::Write;

pub async fn run(
    agent: AgentLoop,
    mut history: History,
    executor: PlainExecutor,
) -> Result<()> {
    // 起動完了を通知
    emit("ready");

    let stdin = std::io::stdin();
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

        history.push(ChatMessage::user(input));

        let mut on_text = |chunk: &str| {
            let msg = json!({"type":"text","delta": chunk});
            println!("{msg}");
            let _ = std::io::stdout().flush();
        };

        match agent.run_turn(&mut history, &executor, &mut on_text).await {
            Ok(_outcome) => emit("ready"),
            Err(e) => {
                let msg = json!({"type":"error","message": e.to_string()});
                println!("{msg}");
                emit("ready");
            }
        }
    }
    Ok(())
}

/// eval ハーネス向け単発実行: 1プロンプトを処理して stdout に結果を出力して終了
pub async fn run_once(
    agent: &AgentLoop,
    history: &mut History,
    executor: &PlainExecutor,
) -> Result<()> {
    let mut full_text = String::new();
    let mut on_text = |chunk: &str| {
        full_text.push_str(chunk);
    };
    match agent.run_turn(history, executor, &mut on_text).await {
        Ok(_) => {
            // eval ハーネスは stdout + stderr を結合して解析するため
            // テキストをそのまま stdout に出力する
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

/// plain モード用 executor: ツール実行前後に JSON Lines を emit する
pub struct PlainExecutor;

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

        let inner = tools::LocalExecutor;
        let result = inner.execute(name, call_id, arguments).await;

        println!("{}", json!({"type":"tool_done","name":name,"ok":!result.is_error}));
        let _ = std::io::stdout().flush();

        result
    }
}
