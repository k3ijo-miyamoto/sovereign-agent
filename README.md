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
    ├── tools/       ツール実装（bash / read_file / write_file / list_files / grep_search / glob_search / edit_file）
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

`eval/` に sovereign バイナリを使って複数のローカルLLMを定量比較するハーネスが含まれている。  
**Phase 0**（バグ修正）と **Phase 1**（実務タスク）の2フェーズで 15 モデルを評価済み。

### 評価の実施方法

```bash
# Phase 0: バグ修正（6ケース）
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# Phase 1: 実務タスク（docstring / テスト生成 / 型アノテーション / コミットメッセージ）
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/run_eval_phase1.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ再実行
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/run_eval.py --model gemma3:27b --cases 04_boundary_bug --no-docker-warn
```

### ログ・結果の保存場所

| パス | 内容 |
|---|---|
| `eval/results_<model>.json` | Phase 0 の各モデルごとの評価結果（自動生成） |
| `eval/results_phase1_<model>.json` | Phase 1 の各モデルごとの評価結果（自動生成） |
| `eval/summary.md` | Phase 0 の全モデル比較サマリ（`python3 eval/summarize.py` で自動生成） |
| `eval/summary_phase1.md` | Phase 0 + Phase 1 の統合サマリ（手動更新） |
| `.sovereign/decisions.jsonl` | sovereign 起動ごとのルーティング判定ログ（JSONL追記） |

### 評価結果サマリ

**実用に耐えるモデル（Phase 0 + Phase 1 両立）:**

| モデル | P0 正解率 | P0 安定性 | P1 正解率 | サイズ |
|---|:---:|:---:|:---:|---:|
| **gemma3:27b** | 5/6 | 94% | 6/6 | 17GB |
| **qwen3:14b** | 5/6 | 94% | 6/6 | 9.3GB |
| **gemma3:12b** | 5/6 | 78% | 6/6 | 8GB |
| qwen3:8b-nothink | 6/6 | 89% | 5/6 | 5.2GB |
| qwen3:8b | 6/6 | 89% | 5/6 | 5.2GB |

> P0 = バグ修正6ケース、P1 = docstring・テスト生成・型アノテーション・コミットメッセージ4タスク

**主な知見:**

- **コード特化モデルが必ずしも勝たない** — `codestral:22b` / `devstral:24b` はエージェント用途での安定性が67%にとどまった
- **gemma3:12b はコスト最優** — docstring・テスト生成・型アノテーションでは 27b と同等スコアを 8GB で達成
- **boundary_bug はほぼ全モデルの壁** — フィボナッチの off-by-one を突破できたのは qwen3 系のみ

**タスク別推奨モデル（`--task` フラグで自動選択される）:**

| `--task` | 推奨モデル | 根拠 |
|---|---|---|
| `docstring` / `type-annotate` | `gemma3:12b` | P1全冠・calls最少 |
| `test` | `qwen3:14b` | P1全冠・安定性高 |
| `commit-msg` | `qwen3:8b-nothink` | 軽量・stab=100% |
| `bugfix` | `gemma3:27b` | P0 安定性94% |

詳細は [eval/summary_phase1.md](eval/summary_phase1.md) および [docs/sovereign-ai.md](docs/sovereign-ai.md) を参照。

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
