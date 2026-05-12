# CLAUDE.md — sovereign-agent

## このプロジェクトについて

ローカルLLM（Ollama）および Anthropic API に対応したクリーンルーム実装のエージェントCLI。
sovereign AI（機密コードを外部APIに送らずにLLMを活用する）の実現を目的とする。

**重要**: このプロジェクトのコードは claw-code リポジトリの既存コードを参照せず、
公式ドキュメントとゼロから書いたクリーンルーム実装である。
コードを追加・修正する際も claw-code の rust/ や src/ を参照しないこと。

## クレート構成

| クレート | 役割 |
|---|---|
| `ollama` | Ollama APIクライアント（ストリーミング・XMLモード対応） |
| `agent` | エージェントループ（モデル非依存・ToolExecutor trait） |
| `tools` | ツール実装（bash / read_file / write_file / list_files） |
| `cli` (`sovereign`) | REPLバイナリエントリポイント |

## ビルドと実行

```bash
# ビルド
cd rust
cargo build -p sovereign

# 実行（Ollamaが必要）
SOVEREIGN_MODEL=gemma3:12b cargo run -p sovereign
SOVEREIGN_MODEL=qwen3:8b  cargo run -p sovereign

# 全クレートビルド確認
cargo build --workspace
```

## XMLモード / エンドポイント選択

Ollama の native tools API 非対応モデルは XML モードで動作する（自動判定）:

| プレフィックス | XMLモード | エンドポイント |
|---|---|---|
| `gemma3`, `phi4`, `devstral` | ✅ | `/api/chat`（ストリーミング） |
| `codestral`, `deepseek`, `mistral-nemo` | ✅ | `/v1/chat/completions`（非ストリーミング・compat） |
| `qwen3`, `qwen2.5` 等 | ❌（native tools API） | `/api/chat`（ストリーミング） |

`codestral` / `deepseek` / `mistral-nemo` は `/api/chat` との相性問題により
`/v1/chat/completions`（OpenAI互換エンドポイント）を使用する（`compat.rs`）。

XMLモードではツール定義をシステムプロンプトに埋め込み、
モデルは `<tool_call>{"name":"...","arguments":{...}}</tool_call>` で応答する。
`<tool_call>` 形式以外（` ```tool_call ```・` ```python ```・生JSON）も自動フォールバックで認識する。

## Eval Harness（ローカルLLM評価）

ソブリンAI（社内の機密コードを外部APIに送らずにLLMを活用する）の実現可能性を検証するための評価基盤。
claw-code で整備し、sovereign-agent に移管したオリジナル実装。`SOVEREIGN_BIN` 環境変数で sovereign バイナリを指定して動作する。

### ディレクトリ構成

```
eval/
├── phase0/
│   ├── run_eval.py       # フェーズ0ハーネス（バグ修正）
│   ├── summarize.py      # フェーズ0結果をMarkdownにまとめるスクリプト
│   ├── summary.md        # フェーズ0評価サマリ（自動生成）
│   ├── cases/            # フェーズ0ケース（buggy.py / expected_output.txt / meta.json）
│   │   ├── 01_syntax_bug/
│   │   ├── 02_type_bug/
│   │   ├── 03_logic_bug/
│   │   ├── 04_boundary_bug/
│   │   ├── 05_state_bug/
│   │   └── 06_safety_trap/
│   └── results/          # フェーズ0モデルごとの評価結果（自動生成）
│       └── <model>.json
├── phase1/
│   ├── run_eval.py       # フェーズ1ハーネス（criteria-based: docstring・テスト生成等）
│   ├── summarize.py      # フェーズ1結果をMarkdownにまとめるスクリプト
│   ├── summary.md        # フェーズ1評価サマリ（自動生成）
│   ├── cases/            # フェーズ1ケース（target.py / prompt.txt / expected_criteria.json）
│   │   ├── 01_docstring_generate/
│   │   ├── 02_docstring_complex/
│   │   ├── 02b_docstring_complex/
│   │   ├── 03_test_generate/
│   │   ├── 04_type_annotate/
│   │   └── 05_commit_message/
│   └── results/          # フェーズ1モデルごとの評価結果（自動生成）
│       └── <model>.json
├── run_eval_docker.sh    # Docker経由実行ラッパー（--phase1 フラグで両フェーズ対応）
└── Dockerfile
```

### 評価の実行

```bash
# --- フェーズ0: バグ修正 ---
# 直接実行
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/phase0/run_eval.py --model gemma3:27b --runs 1 --no-docker-warn

# 安定性評価（選抜モデルのみ）
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/phase0/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ再実行（既存JSONにマージされる）
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/phase0/run_eval.py --model gemma3:27b --cases 04_boundary_bug --no-docker-warn

# サマリ生成
python3 eval/phase0/summarize.py -o eval/phase0/summary.md

# --- フェーズ1: 実務タスク（criteria-based） ---
# 全ケース実行
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/phase1/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/phase1/run_eval.py --model gemma3:27b --cases 03_test_generate --runs 3 --no-docker-warn

# 全モデルを直列実行（Ollamaはシングルスレッドのため並列不可）
export SOVEREIGN_BIN=$(pwd)/rust/target/debug/sovereign
for model in gemma3:27b qwen3:14b qwen3:8b-nothink gemma3:12b qwen3:8b phi4:14b; do
  python3 eval/phase1/run_eval.py --model "$model" --runs 3 --no-docker-warn
done

# サマリ生成
python3 eval/phase1/summarize.py -o eval/phase1/summary.md
```

### フェーズ0評価フロー

2段階で実施：
1. **単回評価**（`--runs 1`）— 全モデルを単回評価し T2 ≥ 5/6 の上位モデルを選抜
2. **安定性評価**（`--runs 3`）— 選抜モデルの安定性（flakiness）を計測

### フェーズ1 チェックメソッド

`expected_criteria.json` の `method` フィールドで指定する：

| method | 説明 |
|---|---|
| `compile` | Python構文エラーがないか（`ast.parse`）|
| `ast` | 関数にdocstringが追加されているか |
| `diff` | 関数本体が変更されていないか（AST比較、docstringを除外）|
| `string_match` | docstring内にキーワードが含まれるか |
| `file_exists` | 指定ファイルが作成されているか |
| `pytest_pass` | `python3 -m pytest <filename>` が exit 0 か |
| `string_match_file` | 指定ファイルの全テキスト内にキーワードが含まれるか |

### 主な知見

最新の数値は `eval/phase0/summary.md` および `eval/phase1/summary.md` を参照。  
タスク別推奨モデルは `README.md` のタスク別推奨モデル表が正（`args.rs` の `task_default_model()` と一致）。

**共通:**
- `gemma3` / `phi4` / `codestral` / `devstral` / `deepseek` は Ollama の tools API 非対応のため XML モードで動作
- `qwen3` 系はネイティブtools APIを使用

### 注意点

- Ollama はシングルスレッドで処理するため並列実行不可。ハーネスは直列実行すること
- `SOVEREIGN_BIN` 環境変数には絶対パスを渡すこと（`Path.resolve()` 済み）。相対パスだと subprocess が tempdir から実行されて見つからない
- `--cwd` フラグはサブコマンドがある場合は機能しないため、ハーネスは `subprocess.run(..., cwd=work_dir)` で作業ディレクトリを指定している
- `--permission-mode danger-full-access` が必須（bash ツールの実行権限が必要）
- フェーズ1の `pytest_pass` チェックには `pip install pytest` が必要

## 自動ルーティング設計方針

設計の詳細・根拠は [docs/sovereign-ai.md のルーティング方針](docs/sovereign-ai.md#ルーティング方針) を参照。

### 実装状況

| 実装ステップ | 内容 | 状態 |
|---|---|---|
| A | タスク種別に応じたデフォルトモデル選択 | ✅ 実装済み（`--task` フラグ・`classify_task()` 自動分類） |
| B | `.sovereign-ai.yml` による機密度ルール | 未実装 |
| C | ルーティング理由の表示・ログ（decision.json） | 未実装 |
| D | LLMによる補助分類（ローカル強制方向のみ） | 後回し |

## Security

```bash
# 脆弱性スキャン（cargo-audit が必要: cargo install cargo-audit）
cargo audit --file rust/Cargo.lock
```
