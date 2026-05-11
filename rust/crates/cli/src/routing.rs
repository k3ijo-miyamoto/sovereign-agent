use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 機密度レベル（数値が大きいほど機密度が高い）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidentiality {
    S0, // 公開情報 — クラウド許可
    S1, // 社内一般 — クラウド許可
    S2, // 機密 — ローカル強制
    S3, // 最重要機密 — ローカル強制
}

impl Confidentiality {
    pub fn requires_local(self) -> bool {
        self >= Confidentiality::S2
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    default_confidentiality: Option<String>,
    paths: Option<HashMap<String, String>>,
}

pub struct SovereignConfig {
    default_level: Confidentiality,
    rules: Vec<(String, Confidentiality)>, // (glob パターン, レベル)
}

impl SovereignConfig {
    /// カレントディレクトリ → ホームディレクトリの順に .sovereign-ai.yml を探す。
    /// 見つからなければ None を返す（デフォルト動作: ローカル強制）。
    pub fn load(cwd: &Path) -> Option<Self> {
        let candidates = [
            cwd.join(".sovereign-ai.yml"),
            dirs_home().map(|h| h.join(".sovereign-ai.yml")).unwrap_or_default(),
        ];
        for path in &candidates {
            if path.exists() {
                if let Ok(text) = std::fs::read_to_string(path) {
                    if let Ok(raw) = serde_yaml::from_str::<RawConfig>(&text) {
                        eprintln!("[routing] config loaded: {}", path.display());
                        return Some(Self::from_raw(raw));
                    }
                }
            }
        }
        None
    }

    fn from_raw(raw: RawConfig) -> Self {
        let default_level = raw.default_confidentiality
            .as_deref()
            .and_then(parse_level)
            .unwrap_or(Confidentiality::S2); // 不明はS2（安全側デフォルト）

        let rules = raw.paths
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(pat, lvl)| parse_level(&lvl).map(|l| (pat, l)))
            .collect();

        Self { default_level, rules }
    }

    /// ファイルパス群の中で最も高い機密度レベルを返す。
    /// ファイルリストが空の場合はデフォルトレベルを返す。
    pub fn classify(&self, files: &[String], cwd: &Path) -> Confidentiality {
        if files.is_empty() {
            return self.default_level;
        }
        files.iter()
            .map(|f| self.classify_one(f, cwd))
            .max()
            .unwrap_or(self.default_level)
    }

    fn classify_one(&self, file: &str, cwd: &Path) -> Confidentiality {
        // ファイルパスをリポジトリルート相対に正規化する
        let rel = relativize(file, cwd);
        for (pat, level) in &self.rules {
            if glob_match(pat, &rel) {
                return *level;
            }
        }
        self.default_level
    }
}

/// シンプルな glob マッチ（`**` と `*` のみ対応）
fn glob_match(pattern: &str, path: &str) -> bool {
    glob_match_inner(pattern.as_bytes(), path.as_bytes())
}

fn glob_match_inner(pat: &[u8], s: &[u8]) -> bool {
    match (pat.first(), s.first()) {
        (None, None) => true,
        (None, _) => false,
        (Some(b'*'), _) => {
            // `**` はパスセパレータを含む任意文字列にマッチ
            if pat.get(1) == Some(&b'*') {
                let rest = if pat.get(2) == Some(&b'/') { &pat[3..] } else { &pat[2..] };
                // ** は空文字列にもマッチ
                if glob_match_inner(rest, s) { return true; }
                for i in 0..=s.len() {
                    if glob_match_inner(rest, &s[i..]) { return true; }
                }
                false
            } else {
                // `*` はパスセパレータ以外の任意文字列にマッチ
                let rest_pat = &pat[1..];
                if glob_match_inner(rest_pat, s) { return true; }
                for i in 0..s.len() {
                    if s[i] == b'/' { break; }
                    if glob_match_inner(rest_pat, &s[i + 1..]) { return true; }
                }
                false
            }
        }
        (Some(&p), Some(&c)) => p == c && glob_match_inner(&pat[1..], &s[1..]),
        _ => false,
    }
}

fn parse_level(s: &str) -> Option<Confidentiality> {
    match s.trim().to_uppercase().as_str() {
        "S0" => Some(Confidentiality::S0),
        "S1" => Some(Confidentiality::S1),
        "S2" => Some(Confidentiality::S2),
        "S3" => Some(Confidentiality::S3),
        _ => None,
    }
}

/// ファイルパスを cwd 相対のスラッシュ区切り文字列に正規化する
fn relativize(file: &str, cwd: &Path) -> String {
    let p = Path::new(file);
    let rel = if p.is_absolute() {
        p.strip_prefix(cwd).unwrap_or(p).to_path_buf()
    } else {
        PathBuf::from(file)
    };
    rel.to_string_lossy().replace('\\', "/")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("src/**", "src/foo/bar.rs"));
        assert!(glob_match("src/**", "src/bar.rs"));
        assert!(glob_match("**/*.py", "a/b/c.py"));
        assert!(glob_match("tests/*", "tests/foo.py"));
        assert!(!glob_match("tests/*", "tests/a/b.py")); // * はセパレータを越えない
        assert!(!glob_match("src/**", "lib/foo.rs"));
    }
}
