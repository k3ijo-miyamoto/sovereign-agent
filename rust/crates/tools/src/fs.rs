use anyhow::{bail, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

const MAX_LINES: usize = 200;

/// パストラバーサル（../）を含むパスを拒否する
fn safe_path(raw: &str) -> Result<&Path> {
    let p = Path::new(raw);
    for component in p.components() {
        if matches!(component, std::path::Component::ParentDir) {
            bail!("パストラバーサルは許可されていません: {raw}");
        }
    }
    Ok(p)
}

pub fn read_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path が必要です"))?;
    let path = safe_path(path_str)?;
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path が必要です"))?;
    let content = args["content"].as_str().ok_or_else(|| anyhow::anyhow!("content が必要です"))?;
    let path = safe_path(path_str)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(format!("書き込み完了: {path_str}"))
}

pub fn list_files(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or(".");
    let path = safe_path(path_str)?;
    let mut entries = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let suffix = if entry.file_type()?.is_dir() { "/" } else { "" };
        entries.push(format!("{name}{suffix}"));
    }
    entries.sort();
    Ok(entries.join("\n"))
}

pub fn grep_search(args: &Value) -> Result<String> {
    let pattern = args["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("pattern が必要です"))?;
    let path_str = args["path"].as_str().unwrap_or(".");
    safe_path(path_str)?;
    let case_insensitive = args["case_insensitive"].as_bool().unwrap_or(false);
    let file_pattern = args["include"].as_str();

    let mut cmd = Command::new("grep");
    cmd.arg("-rn").arg("--color=never");
    if case_insensitive {
        cmd.arg("-i");
    }
    if let Some(fp) = file_pattern {
        cmd.arg(format!("--include={fp}"));
    }
    cmd.arg(pattern).arg(path_str);

    let out = cmd.output()?;
    let raw = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = raw.lines().take(MAX_LINES).collect();
    let truncated = raw.lines().count() > MAX_LINES;
    let mut result = lines.join("\n");
    if truncated {
        result.push_str(&format!("\n... ({}行以上のため打ち切り)", MAX_LINES));
    }
    if result.is_empty() {
        Ok("マッチなし".into())
    } else {
        Ok(result)
    }
}

pub fn glob_search(args: &Value) -> Result<String> {
    let pattern = args["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("pattern が必要です"))?;
    let path_str = args["path"].as_str().unwrap_or(".");
    safe_path(path_str)?;

    let out = Command::new("find")
        .arg(path_str)
        .arg("-name")
        .arg(pattern)
        .arg("-not")
        .arg("-path")
        .arg("*/.*/*")  // 隠しディレクトリ配下をスキップ
        .output()?;

    let raw = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = raw.lines().take(MAX_LINES).collect();
    let truncated = raw.lines().count() > MAX_LINES;
    let mut result = lines.join("\n");
    if truncated {
        result.push_str(&format!("\n... ({}件以上のため打ち切り)", MAX_LINES));
    }
    if result.is_empty() {
        Ok("マッチなし".into())
    } else {
        Ok(result)
    }
}

pub fn edit_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path が必要です"))?;
    let old_str = args["old_string"].as_str().ok_or_else(|| anyhow::anyhow!("old_string が必要です"))?;
    let new_str = args["new_string"].as_str().ok_or_else(|| anyhow::anyhow!("new_string が必要です"))?;
    let path = safe_path(path_str)?;

    let content = std::fs::read_to_string(path)?;
    let count = content.matches(old_str).count();
    if count == 0 {
        bail!("old_string がファイル内に見つかりません: {path_str}");
    }
    if count > 1 {
        bail!("old_string が{count}箇所に存在します。一意になるよう前後の文脈を含めてください");
    }
    let updated = content.replacen(old_str, new_str, 1);
    std::fs::write(path, &updated)?;
    Ok(format!("編集完了: {path_str}"))
}
