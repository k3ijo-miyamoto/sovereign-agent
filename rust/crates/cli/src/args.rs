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

        Self { model, provider, base_url, plain_output, prompt, cwd, allowed_tools, vision_model }
    }
}
