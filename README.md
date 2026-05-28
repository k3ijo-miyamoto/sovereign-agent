# sovereign-agent

> [English](#english) | [日本語](#日本語)

---

## English

An agent CLI that supports both local LLMs (via Ollama) and the Anthropic API.
A clean-room implementation aimed at realizing **Sovereign AI** — leveraging LLMs without sending confidential code to external clouds.

Built from scratch by referencing only the public documentation of the Ollama API, Anthropic Messages API, VS Code Extension API, and Rust libraries. See [LEGAL.md](LEGAL.md) for details.

### Features

- **Fully local operation** — Runs models locally via Ollama. Code never leaves your machine
- **Anthropic API support** — Switchable when you want to use the cloud
- **Automatic XML-mode switching** — Models that do not support the native tools API (gemma3, phi4, codestral, etc.) automatically fall back to XML mode
- **VSCode extension included** — Chat UI usable inside the editor
- **Evaluation harness included** — Quantitatively compares performance and stability across multiple models

### Why "Sovereign AI"

Here, *sovereign* does not merely mean "using a local LLM."
It means keeping control on your side over: where data goes, where the model runs, what routing decisions are made on, how evaluation is conducted, and whether decisions are auditable.

- **Data**: Confidential code is not sent to external APIs
- **Execution**: Runs on a user-managed environment via Ollama
- **Routing**: Local vs. cloud selection is governed by explicit policy
- **Evaluation**: Measured on your real tasks, not generic benchmarks
- **Auditability**: Routing decisions are recorded in `.sovereign/decisions.jsonl`

### How This Differs from Typical Benchmarks

Most LLM benchmarks follow a "ask a question, score the answer" format.
This harness takes a fundamentally different approach. **It launches the `sovereign` CLI as a subprocess against a real temporary filesystem, running it under the same conditions as production.**

```
Harness
  └─ Launches sovereign as a subprocess (temp directory, real files)
       └─ LLM invokes tools: read_file → write_file → bash (verify execution)
            └─ Harness judges: Was the file changed? Does it work correctly?
```

Three things typical benchmarks fail to capture:

- **Measures the full agent loop** — The model must read, fix, and verify via actual tool calls. No API shortcuts
- **Measures reproducibility (not single-shot correctness)** — Each task is run 3 independent times. A model that passes only 1 of 3 is judged "unstable" regardless of score
- **Evaluates minimum intervention** — Beyond correctness, not touching extraneous code is also a pass/fail criterion

A counter-intuitive finding: **code-specialized models (codestral, devstral) lost to general-purpose models.** The reason: "they over-edit and can't stop." See [docs/sovereign-ai.md](docs/sovereign-ai.md) for details.

### Requirements

- Rust 1.75+ (with `cargo` available)
- [Ollama](https://ollama.com/) — local LLM execution engine
- VSCode 1.85+ (if using the extension)

### Quick Start

```bash
# 1. Pull a model with Ollama
ollama pull qwen3:8b

# 2. Build
cd rust
cargo build -p sovereign --release

# 3. Run
SOVEREIGN_MODEL=qwen3:8b cargo run -p sovereign
```

To use the Anthropic API:

```bash
SOVEREIGN_PROVIDER=anthropic ANTHROPIC_API_KEY=sk-... SOVEREIGN_MODEL=claude-sonnet-4-6 cargo run -p sovereign
```

### Crate Layout

```
rust/
└── crates/
    ├── ollama/      Ollama API client (streaming, XML-mode support)
    ├── anthropic/   Anthropic API client
    ├── agent/       Agent loop (model-agnostic, ToolExecutor trait)
    ├── tools/       Tool implementations (bash / read_file / write_file / list_files / grep_search / glob_search / edit_file)
    ├── common/      Shared type definitions
    └── cli/         REPL binary (sovereign)
```

### XML Mode

The following model prefixes do not support Ollama's native tools API and therefore run in XML mode, where tool definitions are embedded in the system prompt. This is transparent to the user — it is auto-detected from the model name.

| XML-mode prefixes |
|---|
| `gemma3`, `phi4`, `codestral`, `devstral`, `deepseek` |

In XML mode, models return tool calls in the following format:

```xml
<tool_call>{"name":"read_file","arguments":{"path":"src/main.rs"}}</tool_call>
```

### VSCode Extension

Installing the extension under `vscode-extension/` adds a side panel in the editor for chatting with the agent.

**Configuration (settings.json):**

| Setting key | Default | Description |
|---|---|---|
| `sovereignAgent.provider` | `ollama` | `ollama` or `anthropic` |
| `sovereignAgent.baseUrl` | `http://localhost:11434` | Ollama endpoint |
| `sovereignAgent.model` | `gemma3:12b` | Model name to use |
| `sovereignAgent.binaryPath` | `auto` | Path to the sovereign binary (`auto` searches automatically) |
| `sovereignAgent.systemPrompt` | — | System prompt appended to the default |

Per-task model overrides (e.g. `sovereignAgent.taskModel.docstring`) are also supported.

### Evaluation Harness

Under `eval/` is a harness that quantitatively compares multiple local LLMs by driving the `sovereign` binary.
15 models have already been evaluated across two phases: **Phase 0** (bug fixing) and **Phase 1** (practical tasks).

#### How to Run an Evaluation

```bash
# Phase 0: Bug fixing (6 cases)
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/phase0/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# Phase 1: Practical tasks (docstring / test generation / type annotation / commit message)
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/phase1/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# Re-run specific cases only
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/phase0/run_eval.py --model gemma3:27b --cases 04_boundary_bug --no-docker-warn
```

#### Log and Result Locations

| Path | Contents |
|---|---|
| `eval/phase0/results/<model>.json` | Per-model Phase 0 results (auto-generated) |
| `eval/phase1/results/<model>.json` | Per-model Phase 1 results (auto-generated) |
| `eval/phase0/summary.md` | Phase 0 cross-model summary (auto-generated by `python3 eval/phase0/summarize.py`) |
| `eval/phase1/summary.md` | Phase 1 cross-model summary (auto-generated by `python3 eval/phase1/summarize.py`) |
| `CHANGELOG.md` | Fix history for the sovereign core (issues found and addressed through eval) |
| `.sovereign/decisions.jsonl` | Per-launch routing decision log (JSONL append) |

#### Evaluation Summary

**Phase 0 — Bug fixing (6 cases)**

A broken program is given to the model; pass/fail is judged by whether the execution output matches the expected value. Each case is run 3 times to measure stability.

| Model | Pass rate | Stability | Size |
|---|:---:|:---:|---:|
| **qwen3:14b** | 6/6 | 100% | 9.3GB |
| **phi4:14b** | 6/6 | 89% | 9.1GB |
| **qwen3:8b** | 6/6 | 86% | 5.2GB |
| **gemma3:12b** | 5/6 | 83% | 8.1GB |
| gemma3:27b | 5/6 | 78% | 17.0GB |
| qwen3:8b-nothink | 5/6 | 78% | 5.2GB |
| devstral:24b | 5/6 | 72% | 14.0GB |
| qwen2.5:7b | 5/6 | 72% | 4.7GB |

> `boundary_bug` (Fibonacci off-by-one) is a wall for nearly every model. Only `deepseek-coder-v2:16b`, `phi4:14b`, `qwen3:14b`, and `qwen3:8b` broke through.

**Phase 1 — Practical tasks (6 cases)**

Tests whether "add / generate"-style tasks are completed correctly *and* minimally, checked against multiple criteria (AST / pytest / keyword match, etc.). Whether the body of the function was left untouched (minimum intervention) is also an evaluation axis.

| Task | Check method |
|---|---|
| Add docstring | AST check that a docstring was inserted and the body was not changed |
| Generate unit tests | `pytest` exits 0 |
| Add type annotations | AST check that annotations were added and the body was not changed |
| Generate commit message | Required keywords are present |

> Docstring tasks are split into 3 sub-cases (simple / complex / hinted). Other tasks have 1 case each.

| Model | Cases passed | Size |
|---|:---:|---:|
| **gemma3:12b** | 6/6 | 8.1GB |
| **gemma3:27b** | 6/6 | 17.0GB |
| **qwen3:14b** | 6/6 | 9.3GB |
| **qwen3:8b** | 6/6 | 5.2GB |
| **qwen2.5-coder:14b** | 6/6 | 9.0GB |
| **devstral:24b** | 6/6 | 14.0GB |
| **phi4:14b** | 6/6 | 9.1GB |
| qwen3:8b-nothink | 5/6 | 5.2GB |

> The top 7 P1 models all pass 6/6. Stability differs sharply, though: gemma3:12b (100%) vs. phi4:14b (50%).

**Key findings:**

- **Only qwen3:14b solved all 6 cases on all 3 runs** — P0 stab=100% / P1 stab=83%. Every other model wavers somewhere. If "must always work" is a requirement, this is currently the only choice (9.3GB)
- **gemma3:12b achieves stab=100% on practical tasks** — All 6 P1 cases pass on every run. With `calls=2.1`, it is also the most efficient. Delivers the same result as 27b at half the size (8.1GB)
- **qwen3:8b sweeps both P0 and P1 at 5.2GB** — Even breaks `boundary_bug` at stab=67%. The only lightweight model that reaches a usable level on RAM-constrained environments
- **`boundary_bug` is the watershed for "logical reading ability"** — 11 of 15 models failed all 3 times. Only qwen3:14b (100%), qwen3:8b (67%), phi4:14b (33%), and deepseek-coder-v2:16b (33%) made it through
- **Toggling `thinking` changes behavior even for the same model** — qwen3:8b-nothink hits stab=0% on `type_annotate` (failing all 3 runs); the thinking-enabled version passes at stab=50%. Switching modes per task is worth doing
- **devstral:24b is good at "generating" but poor at "fixing"** — Sweeps P1 (docstring / test / type annotation addition) at 6/6, yet hits stab=0% on the P0 `boundary_bug`. Falls apart on complex logical bug fixes

**Recommended models per task (auto-selected via the `--task` flag):**

| `--task` | Recommended model | Rationale |
|---|---|---|
| `docstring` / `type-annotate` | `gemma3:12b` | P1 sweep / stab=100% / calls=2.1 |
| `test` | `gemma3:12b` | P1 sweep / stab=100% / only model passing `covers_partial` |
| `commit-msg` | `qwen3:8b-nothink` | Lightweight / commit_message stab=100% |
| `bugfix` | `qwen3:14b` | P0 stab=100% / boundary_bug 100% |

> On lightweight environments (~5GB), override with `--model qwen3:8b` (sweeps P0 and P1, boundary_bug stab=67%).

For details and a recommended reading order, see [docs/sovereign-ai.md](docs/sovereign-ai.md#読む順番).

### Security

- `unsafe_code = "forbid"` — `unsafe` blocks are forbidden workspace-wide
- Dependencies are pinned via `Cargo.lock`. Non-crates.io sources are rejected by `cargo deny`
- `cargo audit` and `cargo deny check` are run weekly on GitHub Actions

### Build

```bash
cd rust

# CLI only
cargo build -p sovereign

# All crates
cargo build --workspace

# Release build
cargo build -p sovereign --release
```

### License

MIT

---

## 日本語

ローカルLLM（Ollama）および Anthropic API に対応したエージェントCLI。
機密コードを外部クラウドに送らずにLLMを活用する **Sovereign AI** の実現を目的としたクリーンルーム実装。

Ollama API・Anthropic Messages API・VS Code Extension API・Rust ライブラリの公開ドキュメントのみを参照してゼロから実装しています。詳細は [LEGAL.md](LEGAL.md) を参照。

## 特徴

- **完全ローカル動作** — Ollama 経由でモデルをローカル実行。コードがクラウドに送出されない
- **Anthropic API にも対応** — クラウドを使いたい場合は切り替え可能
- **XMLモード自動切換** — native tools API 非対応モデル（gemma3, phi4, codestral 等）は自動でXMLモードに切り替え
- **VSCode 拡張付属** — チャットUIをエディタ内で使用可能
- **評価ハーネス付属** — 複数モデルの性能・安定性を定量比較できる

## なぜ「Sovereign AI」か

ここでの sovereign は、単にローカルLLMを使うことを意味しない。  
データの行き先、モデルの実行場所、ルーティングの判断基準、評価の方法、そして判断の監査可能性 — それらの制御権を自分たちの側に置くことを意味する。

- **データ**: 機密コードを外部APIに送らない
- **実行**: Ollama 経由でユーザー管理環境上で動く
- **ルーティング**: ローカル/クラウドの選択は明示的なポリシーに基づく
- **評価**: 一般ベンチマークではなく自分たちの実タスクで測る
- **監査**: ルーティング判断を `.sovereign/decisions.jsonl` に記録する

## 一般的なベンチマークとの違い

LLM の多くのベンチマークは「質問して回答を採点する」形式です。  
このハーネスはそれとは根本的に異なる方法を取っています。**sovereign CLI を subprocess として本物の一時ファイルシステムに対して起動し、本番と同じ条件で動かします**。

```
ハーネス
  └─ sovereign を subprocess として起動（一時ディレクトリ・本物のファイル）
       └─ LLM がツールを呼び出す: read_file → write_file → bash（実行確認）
            └─ ハーネスが判定: ファイルは変更されたか？正しく動くか？
```

典型的なベンチマークでは捉えられない3つのポイント：

- **エージェントループの全体を測る** — モデルは実際のツール呼び出しで読み・直し・確認しなければならない。API ショートカットはない
- **再現性を測る（1回の正解ではなく）** — 各タスクを3回独立実行。3回中1回しか通らないモデルはスコアに関わらず「不安定」と判定
- **最小介入を評価する** — 正解するだけでなく、余計なコードを変えていないことも合否条件に含まれる

反直感的な発見：**コード特化モデル（codestral, devstral）が汎用モデルに負けた**。理由は「過剰に編集して止まれない」こと。詳細は [docs/sovereign-ai.md](docs/sovereign-ai.md) を参照。

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
  python3 eval/phase0/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# Phase 1: 実務タスク（docstring / テスト生成 / 型アノテーション / コミットメッセージ）
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/phase1/run_eval.py --model gemma3:27b --runs 3 --no-docker-warn

# 特定ケースのみ再実行
SOVEREIGN_BIN=../rust/target/debug/sovereign \
  python3 eval/phase0/run_eval.py --model gemma3:27b --cases 04_boundary_bug --no-docker-warn
```

### ログ・結果の保存場所

| パス | 内容 |
|---|---|
| `eval/phase0/results/<model>.json` | Phase 0 の各モデルごとの評価結果（自動生成） |
| `eval/phase1/results/<model>.json` | Phase 1 の各モデルごとの評価結果（自動生成） |
| `eval/phase0/summary.md` | Phase 0 の全モデル比較サマリ（`python3 eval/phase0/summarize.py` で自動生成） |
| `eval/phase1/summary.md` | Phase 1 の全モデル比較サマリ（`python3 eval/phase1/summarize.py` で自動生成） |
| `CHANGELOG.md` | sovereign 本体への修正履歴（eval 実施を通じて発見した問題と対処） |
| `.sovereign/decisions.jsonl` | sovereign 起動ごとのルーティング判定ログ（JSONL追記） |

### 評価結果サマリ

**Phase 0 — バグ修正（6ケース）**

壊れたコードを渡し、実行出力が期待値と一致するかで合否を判定。6ケースを3回ずつ実行して安定性を計測。

<!-- eval-p0-start -->
| モデル | 正解率 | 安定性 | サイズ |
|---|:---:|:---:|---:|
| **qwen3:14b** | 6/6 | 100% | 9.3GB |
| **phi4:14b** | 6/6 | 89% | 9.1GB |
| **qwen3:8b** | 6/6 | 86% | 5.2GB |
| **gemma3:12b** | 5/6 | 83% | 8.1GB |
| gemma3:27b | 5/6 | 78% | 17.0GB |
| qwen3:8b-nothink | 5/6 | 78% | 5.2GB |
| devstral:24b | 5/6 | 72% | 14.0GB |
| qwen2.5:7b | 5/6 | 72% | 4.7GB |

> boundary_bug（フィボナッチ off-by-one）はほぼ全モデルの壁。突破できたのは `deepseek-coder-v2:16b`, `phi4:14b`, `qwen3:14b`, `qwen3:8b` のみ。
<!-- eval-p0-end -->

**Phase 1 — 実務タスク（6ケース）**

「追加・生成」系のタスクを正しく・最小限にこなせるかを複数の基準（AST / pytest / キーワードマッチ等）でチェック。本体を変更していないか（最小介入）も評価軸に含む。

| タスク | 評価方法 |
|---|---|
| docstring 追加 | AST で docstring が挿入されているか・本体が変わっていないか |
| ユニットテスト生成 | `pytest` が pass するか |
| 型アノテーション追加 | AST で型アノテーションが付いているか・本体が変わっていないか |
| コミットメッセージ生成 | 必須キーワードが含まれているか |

<!-- eval-p1-start -->
> docstring 追加は3サブケース（単純・複雑・ヒント付き）に分けて評価。他タスクは各1ケース。

| モデル | ケース通過 | サイズ |
|---|:---:|---:|
| **gemma3:12b** | 6/6 | 8.1GB |
| **gemma3:27b** | 6/6 | 17.0GB |
| **qwen3:14b** | 6/6 | 9.3GB |
| **qwen3:8b** | 6/6 | 5.2GB |
| **qwen2.5-coder:14b** | 6/6 | 9.0GB |
| **devstral:24b** | 6/6 | 14.0GB |
| **phi4:14b** | 6/6 | 9.1GB |
| qwen3:8b-nothink | 5/6 | 5.2GB |
<!-- eval-p1-end -->

> P1 上位7モデルがすべて 6/6 通過。ただし安定性には差があり、gemma3:12b（100%）と phi4:14b（50%）では大きく異なる。

**主な知見:**

<!-- insights-start -->
- **qwen3:14b だけが全6ケースを3回とも解けた** — P0 stab=100%・P1 stab=83%。他モデルは必ずどこかで揺れる。「必ず動く」が必要なら現状唯一の選択肢（9.3GB）
- **gemma3:12b は実務タスクで stab=100%** — P1 全6ケース3回とも通過。calls=2.1 と手数も最少。27b と同等の成果を半分のサイズ（8.1GB）で出す
- **qwen3:8b は 5.2GB で P0・P1 ともに全冠** — 境界値バグも stab=67% で突破。RAM が限られる環境でも実用レベルに到達した唯一の軽量モデル
- **boundary_bug は「論理を読む力」の分水嶺** — 15 モデル中 11 が 3 回とも失敗。突破できたのは qwen3:14b（100%）・qwen3:8b（67%）・phi4:14b（33%）・deepseek-coder-v2:16b（33%）のみ
- **thinking の有無で同じモデルの挙動が変わる** — qwen3:8b-nothink は type_annotate stab=0%（3 回全滅）、thinking あり版は stab=50% で通過。タスクによってモードを使い分ける価値がある
- **devstral:24b は「生成」は得意・「修正」は不得意** — P1（docstring/テスト/型アノテーション追加）は 6/6 全冠だが、P0 の boundary_bug は stab=0%。複雑な論理バグ修正では崩れる
<!-- insights-end -->

**タスク別推奨モデル（`--task` フラグで自動選択される）:**

| `--task` | 推奨モデル | 根拠 |
|---|---|---|
| `docstring` / `type-annotate` | `gemma3:12b` | P1全冠・stab=100%・calls=2.1 |
| `test` | `gemma3:12b` | P1全冠・stab=100%・covers_partial 唯一通過 |
| `commit-msg` | `qwen3:8b-nothink` | 軽量・commit_message stab=100% |
| `bugfix` | `qwen3:14b` | P0 stab=100%・boundary_bug 100% |

> 軽量環境（〜5GB）では `--model qwen3:8b` で上書き（P0・P1 全冠・boundary_bug stab=67%）。

詳細・ドキュメントの読む順番は [docs/sovereign-ai.md](docs/sovereign-ai.md#読む順番) を参照。

## セキュリティ

- `unsafe_code = "forbid"` — ワークスペース全体で unsafe ブロックを禁止
- 依存関係は `Cargo.lock` で固定済み。crates.io 以外のソースは `cargo deny` で拒否
- `cargo audit` および `cargo deny check` を GitHub Actions で週次自動実行

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
