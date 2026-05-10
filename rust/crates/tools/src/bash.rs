use anyhow::{bail, Result};
use serde_json::Value;
use tokio::process::Command;

pub async fn run(args: &Value) -> Result<String> {
    let cmd = args["command"].as_str().ok_or_else(|| anyhow::anyhow!("command が必要です"))?;

    // 危険なパターンを拒否
    for banned in &["rm -rf /", ":(){ :|:& };:", "dd if=/dev/"] {
        if cmd.contains(banned) {
            bail!("拒否されたコマンドパターンです: {banned}");
        }
    }

    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.into_owned())
    } else {
        Ok(format!("exit {}\n{stderr}", output.status.code().unwrap_or(-1)))
    }
}
