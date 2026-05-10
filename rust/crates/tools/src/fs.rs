use anyhow::{bail, Result};
use serde_json::Value;
use std::path::Path;

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
