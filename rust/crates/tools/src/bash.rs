use anyhow::{bail, Result};
use serde_json::Value;
use tokio::process::Command;

/// unshare(1) が使えるか起動時に一度だけ確認する
static UNSHARE_OK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn has_unshare() -> bool {
    *UNSHARE_OK.get_or_init(|| {
        // user namespace が unprivileged で使えるか確認 (man 1 unshare)
        std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "true"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

/// bash コマンドを名前空間で隔離して実行する。
/// 隔離できない環境（Docker 内など）では直接実行にフォールバックする。
fn build_command(cmd: &str) -> Command {
    if has_unshare() {
        // ユーザー名前空間 + マウント名前空間 + PID 名前空間で隔離
        // 参照: man 1 unshare, man 7 namespaces, man 7 user_namespaces
        let mut c = Command::new("unshare");
        c.args(["--user", "--map-root-user", "--mount", "--pid", "--fork", "--kill-child", "--"])
         .arg("bash").arg("-c").arg(cmd);
        c
    } else {
        let mut c = Command::new("bash");
        c.arg("-c").arg(cmd);
        c
    }
}

pub async fn run(args: &Value) -> Result<String> {
    let cmd = args["command"].as_str().ok_or_else(|| anyhow::anyhow!("command が必要です"))?;

    for banned in &["rm -rf /", ":(){ :|:& };:", "dd if=/dev/"] {
        if cmd.contains(banned) {
            bail!("拒否されたコマンドパターンです: {banned}");
        }
    }

    let output = build_command(cmd).output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.into_owned())
    } else {
        Ok(format!("exit {}\n{stderr}", output.status.code().unwrap_or(-1)))
    }
}
