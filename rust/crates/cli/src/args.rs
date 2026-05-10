/// CLI 引数パーサー（std のみで実装、依存ゼロ）
pub struct Cli {
    pub model: String,
    pub provider: String,
    pub base_url: String,
    pub plain_output: bool,
    /// `prompt <text>` サブコマンド（単発実行）
    pub prompt: Option<String>,
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

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "--plain-output" | "--plain" => {
                    plain_output = true;
                }
                "--model" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { model = v.clone(); }
                }
                "--provider" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { provider = v.clone(); }
                }
                "--base-url" => {
                    i += 1;
                    if let Some(v) = raw.get(i) { base_url = v.clone(); }
                }
                // eval ハーネス互換: 受け付けるが無視するフラグ
                "--permission-mode" => { i += 1; } // 常に full access
                // `prompt <text>` サブコマンド
                "prompt" => {
                    // 残りの引数すべてをプロンプトとして結合
                    let rest = raw[i + 1..].join(" ");
                    prompt = Some(rest);
                    break;
                }
                _ => {}
            }
            i += 1;
        }

        Self { model, provider, base_url, plain_output, prompt }
    }
}
