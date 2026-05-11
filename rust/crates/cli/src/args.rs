/// CLI 引数パーサー（std のみで実装、依存ゼロ）
pub struct Cli {
    pub model: String,
    pub provider: String,
    pub base_url: String,
    pub plain_output: bool,
    pub prompt: Option<String>,
    pub cwd: Option<String>,
    pub allowed_tools: Option<String>, // カンマ区切り
    pub vision_model: Option<String>,
    /// --task で指定されたタスク種別（モデル自動選択に使用）
    pub task: Option<String>,
    /// --task によってモデルが自動選択されたか（ログ表示用）
    pub model_from_task: bool,
    /// ルールベース自動分類によってタスクが推定されたか（ログ表示用）
    pub task_auto_detected: bool,
}

/// タスク種別からデフォルトモデルを返す（eval 実測ベース）
pub fn task_default_model(task: &str) -> Option<&'static str> {
    match task {
        "docstring" | "type-annotate" => Some("gemma3:12b"),
        "test"                         => Some("qwen3:14b"),
        "commit-msg"                   => Some("qwen3:8b-nothink"),
        "bugfix"                       => Some("gemma3:27b"),
        _                              => None,
    }
}

/// プロンプトテキストからタスク種別をルールベースで推定する。
/// 判定できない場合は None を返す（デフォルトモデルにフォールバック）。
pub fn classify_task(prompt: &str) -> Option<&'static str> {
    let lower = prompt.to_lowercase();
    // 優先度順にマッチ（より具体的なものを先に）
    if contains_any(&lower, &["type hint", "type annotation", "型アノテーション", "型ヒント", "型を付"]) {
        return Some("type-annotate");
    }
    if contains_any(&lower, &["docstring", "ドキュメント", "説明を書", "document"]) {
        return Some("docstring");
    }
    if contains_any(&lower, &["commit message", "コミットメッセージ", "コミットメッセ", "commit msg"]) {
        return Some("commit-msg");
    }
    if contains_any(&lower, &["pytest", "unittest", "テストコード", "テストを書", "テストを生成", "test case", "write test", "generate test"]) {
        return Some("test");
    }
    if contains_any(&lower, &["バグ", "bug", "エラー", "error", "修正して", "直して", "fix"]) {
        return Some("bugfix");
    }
    None
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}

impl Cli {
    pub fn parse() -> Self {
        let raw: Vec<String> = std::env::args().skip(1).collect();
        Self::from_raw(&raw)
    }

    fn from_raw(raw: &[String]) -> Self {
        let mut model = std::env::var("SOVEREIGN_MODEL").unwrap_or_else(|_| "gemma3:12b".into());
        let mut provider =
            std::env::var("SOVEREIGN_PROVIDER").unwrap_or_else(|_| "ollama".into());
        let mut base_url =
            std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into());
        let mut plain_output = false;
        let mut prompt: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut allowed_tools: Option<String> = None;
        let mut vision_model: Option<String> = None;
        let mut task: Option<String> = None;
        let mut model_explicit = false;

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "--plain-output" | "--plain" => {
                    plain_output = true;
                }
                "--model" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { model = v.clone(); model_explicit = true; }
                }
                "--task" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { task = Some(v.clone()); }
                }
                "--provider" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { provider = v.clone(); }
                }
                "--base-url" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { base_url = v.clone(); }
                }
                "--cwd" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { cwd = Some(v.clone()); }
                }
                "--allowed-tools" | "--allowedTools" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { allowed_tools = Some(v.clone()); }
                }
                "--vision-model" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { vision_model = Some(v.clone()); }
                }
                // eval ハーネス互換: 受け付けるが無視するフラグ
                "--permission-mode" => { i += 1; }
                // `prompt <text>` サブコマンド
                "prompt" => {
                    let rest = raw[i + 1..].join(" ");
                    prompt = Some(rest);
                    break;
                }
                _ => {}
            }
            i += 1;
        }

        // --task が未指定 かつ prompt が確定している場合、ルールベースで自動分類する
        let mut task_auto_detected = false;
        if task.is_none() && !model_explicit {
            if let Some(ref p) = prompt {
                if let Some(detected) = classify_task(p) {
                    task = Some(detected.to_string());
                    task_auto_detected = true;
                }
            }
        }

        // タスクが確定していて --model が明示されていない場合、タスクからモデルを選択する
        let model_from_task = !model_explicit;
        if let Some(ref t) = task {
            if !model_explicit {
                if let Some(m) = task_default_model(t) {
                    model = m.to_string();
                }
            }
        }

        Self { model, provider, base_url, plain_output, prompt, cwd, allowed_tools, vision_model, task, model_from_task, task_auto_detected }
    }
}
