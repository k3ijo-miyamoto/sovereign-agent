# sovereign-agent

ローカルLLM（Ollama）および Anthropic API に対応したエージェントCLI。
機密コードを外部クラウドに送らずにLLMを活用する **Sovereign AI** の実現を目的としたクリーンルーム実装。

## 特徴

- **完全ローカル動作** — Ollama 経由でモデルをローカル実行。コードがクラウドに送出されない
- **Anthropic API にも対応** — クラウドを使いたい場合は切り替え可能
- **XMLモード自動切換** — native tools API 非対応モデル（gemma3, phi4, codestral 等）は自動でXMLモードに切り替え
- **VSCode 拡張付属** — チャットUIをエディタ内で使用可能
- **評価ハーネス付属** — 複数モデルの性能・安定性を定量比較できる

## 必要環境

- Rust 1.75+（`cargo` コマンドが使えること）
- [Ollama](https://ollama.com/) — ローカルLLM実行エンジン
- VSCode 1.85+（拡張機能を使う場合）

## クイックスタート

```bash
# 1. Ollama でモデルを取得
ollama pull qwen3:8b

# 2. ビルド
cd rust
cargo build -p sovereign --release

# 3. 起動
SOVEREIGN_MODEL=qwen3:8b cargo run -p sovereign
```

Anthropic API を使う場合:

```bash
SOVEREIGN_PROVIDER=anthropic ANTHROPIC_API_KEY=sk-... SOVEREIGN_MODEL=claude-sonnet-4-6 cargo run -p sovereign
```

## クレート構成

```
rust/
└── crates/
    ├── ollama/      Ollama APIクライアント（ストリーミング・XMLモード対応）
    ├── anthropic/   Anthropic APIクライアント
    ├── agent/       エージェントループ（モデル非依存・ToolExecutor trait）
    ├── tools/       ツール実装（bash / read_file / write_file / list_files）
    ├── common/      共有型定義
    └── cli/         REPLバイナリ（sovereign）
```

## XMLモード

以下のモデルプレフィックスは Ollama の native tools API 非対応のため、ツール定義をシステムプロンプトに埋め込むXMLモードで動作する。ユーザーが意識する必要はなく、モデル名から自動判定される。

| XMLモード対象プレフィックス |
|---|
| `gemma3`, `phi4`, `codestral`, `devstral`, `deepseek` |

モデルは以下の形式でツール呼び出しを返す:

```xml
<tool_call>{"name":"read_file","arguments":{"path":"src/main.rs"}}</tool_call>
```

## VSCode 拡張

`vscode-extension/` に含まれる拡張機能をインストールすると、エディタ内のサイドパネルでエージェントとチャットできる。

**設定項目（settings.json）:**

| 設定キー | デフォルト | 説明 |
|---|---|---|
| `sovereignAgent.provider` | `ollama` | `ollama` または `anthropic` |
| `sovereignAgent.baseUrl` | `http://localhost:11434` | Ollama のエンドポイント |
| `sovereignAgent.model` | `gemma3:12b` | 使用するモデル名 |
| `sovereignAgent.binaryPath` | `auto` | sovereign バイナリのパス（autoで自動検索） |
| `sovereignAgent.systemPrompt` | — | デフォルトに追記するシステムプロンプト |

タスク別モデルの個別指定（`sovereignAgent.taskModel.docstring` 等）も可能。

## 評価ハーネス

`eval/` にローカルLLMの性能・安定性を定量評価するハーネスが含まれている。

```bash
# 全モデル単回評価
cd eval
python3 run_eval.py --model gemma3:27b --no-docker-warn

# フェーズ1評価（非コード編集タスク）
python3 run_eval_phase1.py --model gemma3:27b --no-docker-warn

# 安定性評価（3回ずつ実行）
python3 run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# バイナリを明示する場合（リリースビルド等）
SOVEREIGN_BIN=../rust/target/release/sovereign python3 run_eval.py --model qwen3:14b --no-docker-warn
```

### 評価結果サマリ（フェーズ2: 安定性評価）

バグ修正6ケースを3回ずつ実行した安定性評価の結果:

| モデル | 正解率 | 安定性 | サイズ |
|---|:---:|:---:|---:|
| **gemma3:27b** | 6/6 | 94% | 17.0GB |
| **qwen3:14b** | 6/6 | 94% | 9.3GB |
| qwen3:8b-nothink | 6/6 | 89% | 5.2GB |
| qwen3:8b | 6/6 | 89% | — |
| phi4:14b | 6/6 | 89% | — |
| gemma3:12b | 5/6 | 78% | — |
| codestral:22b | 5/6 | 67% | — |
| devstral:24b | 5/6 | 67% | — |

**推薦モデル:**
- 精度・安定性最優先 → `gemma3:27b`
- バランス重視 → `qwen3:14b`（9.3GB、精度・安定性ともに最高水準）
- 軽量環境（〜5GB） → `qwen3:8b-nothink`

詳細は [eval/summary.md](eval/summary.md) を参照。

## ビルド

```bash
cd rust

# CLIのみ
cargo build -p sovereign

# 全クレート
cargo build --workspace

# リリースビルド
cargo build -p sovereign --release
```

## ライセンス

MIT
