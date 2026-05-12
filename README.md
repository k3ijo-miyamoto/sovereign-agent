# sovereign-agent

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

詳細は [docs/sovereign-ai.md](docs/sovereign-ai.md) を参照。

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
