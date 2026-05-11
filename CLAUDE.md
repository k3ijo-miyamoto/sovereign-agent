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
├── run_eval.py           # フェーズ0ハーネス（バグ修正）
├── run_eval_phase1.py    # フェーズ1ハーネス（criteria-based: docstring・テスト生成等）
├── summarize.py          # フェーズ0結果をMarkdownにまとめるスクリプト
├── summary.md            # フェーズ0評価サマリ（自動生成）
├── summary_phase1.md     # フェーズ1評価サマリ（手動更新）
├── cases/                # フェーズ0ケース（buggy.py / expected_output.txt / meta.json）
│   ├── 01_syntax_bug/
│   ├── 02_type_bug/
│   ├── 03_logic_bug/
│   ├── 04_boundary_bug/
│   ├── 05_state_bug/
│   └── 06_safety_trap/
├── cases_phase1/         # フェーズ1ケース（target.py / prompt.txt / expected_criteria.json）
│   ├── 01_docstring_generate/   # normalize_scores — min-max正規化
│   ├── 02_docstring_complex/    # compute_moving_average — 引数5つの複雑な関数
│   └── 03_test_generate/        # chunk — pytestで自動検証
└── results_<model>.json  # フェーズ0モデルごとの評価結果（自動生成）
```

### 評価の実行

```bash
# --- フェーズ0: バグ修正 ---
# 直接実行
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/run_eval.py --model gemma3:27b --runs 1 --no-docker-warn

# 安定性評価（選抜モデルのみ）
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ再実行（既存JSONにマージされる）
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/run_eval.py --model gemma3:27b --cases 04_boundary_bug --no-docker-warn

# サマリ生成
python3 eval/summarize.py -o eval/summary.md

# --- フェーズ1: 実務タスク（criteria-based） ---
# 全ケース実行
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/run_eval_phase1.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ
SOVEREIGN_BIN=rust/target/debug/sovereign python3 eval/run_eval_phase1.py --model gemma3:27b --cases 03_test_generate --runs 3 --no-docker-warn

# 全モデルを直列実行（Ollamaはシングルスレッドのため並列不可）
export SOVEREIGN_BIN=/path/to/sovereign-agent/rust/target/debug/sovereign
for model in gemma3:27b qwen3:14b qwen3:8b-nothink gemma3:12b qwen3:8b phi4:14b; do
  python3 eval/run_eval_phase1.py --model "$model" --runs 3 --no-docker-warn
done
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

### 主な知見（2026-05、claw-code バイナリで計測）

**フェーズ0（バグ修正）:**
- **第1推薦**: `gemma3:27b` — T2=6/6、安定性94%、calls=3.0（最も効率的）
- **第2推薦**: `qwen3:14b` — T2=6/6、安定性94%、T3も完全（6/6）
- **軽量向け**: `qwen3:8b-nothink` — 5GBでT2=6/6、安定性89%
- `devstral:24b` / `codestral:22b` は単回評価では上位だが安定性67%（boundary_bug で0%）

**フェーズ1（docstring生成）:**
- `gemma3:12b` が `gemma3:27b` と完全同等スコア（docstringタスクに限り）
- `qwen3:8b` がcalls=2.2・stab=100%でTier1入り — サイズに関係なく安定
- `codestral:22b` / `deepseek-coder-v2:16b` 等コード特化モデルが逆に失速（本体変更・calls増大）
- `granite3.3:8b` は calls=54.3（ループ状態）、`llama3.1:8b` は syntax_valid❌

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

ローカルLLMをどのタスク・どのファイルに使うかを自動判断する仕組み。2軸・4フェーズで実装する。

### 軸1: タスク → モデル選択（Phase A）

タスク種別に応じて eval 実測ベースのデフォルトモデルを選択する。

| タスク | デフォルトモデル |
|---|---|
| docstring / 型アノテーション | gemma3:12b |
| テスト生成 | qwen3:14b |
| コミットメッセージ | qwen3:8b-nothink |
| バグ修正（複雑） | gemma3:27b |

### 軸2: 機密度 → ローカル/クラウド選択（Phase B）

`.sovereign-ai.yml`（リポジトリルートまたはホームディレクトリ）でパスベースのルールを定義する。

```yaml
default_confidentiality: S2   # 不明はS2扱い（安全側デフォルト）
paths:
  "customer/**": S3
  "internal/**": S2
  "src/business_logic/**": S3
  "docs/**": S0
  "tests/**": S1
```

**設計原則:**
- 判定不能・迷ったら → ローカル固定（false negative が致命的なため）
- LLMによる機密度分類は「ローカル強制の補助」にのみ使う。クラウド許可の根拠にしない

### 実装ロードマップ

| Phase | 内容 | 状態 |
|---|---|---|
| A | タスク種別に応じたデフォルトモデル選択 | 未実装 |
| B | `.sovereign-ai.yml` による機密度ルール | 未実装 |
| C | ルーティング理由の表示・ログ（decision.json） | 未実装 |
| D | LLMによる補助分類（ローカル強制方向のみ） | 後回し |

## Security

```bash
# 脆弱性スキャン（cargo-audit が必要: cargo install cargo-audit）
cargo audit --file rust/Cargo.lock
```
