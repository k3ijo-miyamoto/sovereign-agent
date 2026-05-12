# フェーズ0 + フェーズ1 評価サマリ（sovereign-agent・全15モデル）

> 更新日: 2026-05-11
> ハーネス: `eval/phase0/run_eval.py`（Phase 0）/ `eval/phase1/run_eval.py`（Phase 1）
> バイナリ: `rust/target/release/sovereign`

> [!NOTE]
> このファイルは **Claude が生成・更新する手動ドキュメント**です。
> 個別数値の一次情報は各フェーズのスクリプトが JSON から自動生成するサマリを参照してください。
>
> | 正確な数値の参照先 | 更新方法 |
> |---|---|
> | Phase 0 詳細 → `eval/phase0/summary.md` | `python3 eval/phase0/summarize.py -o eval/phase0/summary.md` |
> | Phase 1 詳細 → `eval/phase1/summary.md` | `python3 eval/phase1/summarize.py -o eval/phase1/summary.md` |
>
> 再評価後にこのファイルの数値が古くなった場合は、上記スクリプトを実行した後、Claude に「eval_results.md を更新して」と依頼してください。

---

## 総括・所見

### 所見1: 実用に耐えるモデルは6つに絞られる

両フェーズで安定して動くのは以下6モデル。

| モデル | P0 T2 | P1 T2 | 特記 |
|---|---|---|---|
| **qwen3:14b** | 5/6 | 6/6 | P1 全冠・P0 boundary_bug のみ未解決 |
| **gemma3:27b** | 5/6 | 6/6 | P1 全冠・P0 boundary_bug のみ未解決 |
| **gemma3:12b** | 5/6 | 6/6 | P1 全冠・27b と同等・コスト優位 |
| **qwen2.5-coder:14b** | 5/6 | 6/6 | P1 全冠・calls やや多め |
| **qwen3:8b-nothink** | 6/6 | 5/6 | P0 完全制覇・P1 type_annotate だけ失敗 |
| **qwen3:8b** | 6/6 | 5/6 | P0 完全制覇・P1 type_annotate だけ失敗 |

### 所見2: boundary_bug はほぼ全モデルの壁

「nth_fibonacci が range(n) を使っておりオフバイワン」という境界値バグは、多くのモデルが「出力は正しく見える」罠にはまって失敗する。解決できたのは **qwen3:8b・qwen3:8b-nothink・phi4:14b・deepseek-coder-v2:16b** の4モデルのみ。

→ 複雑なロジックバグは qwen3:8b 系が最も信頼できる（deepseek-coder-v2:16b も解決可能だが P1 の安定性が低い）。

### 所見3: コスト最適解は gemma3:12b（フェーズ1限定）

フェーズ1（docstring・テスト生成・型アノテーション・コミットメッセージ）では、gemma3:12b が 6/6 かつ calls=1.8（最軽量クラス）を達成。モデルサイズが27b の半分でほぼ同等の成果を出している。

→ **実務タスク全般**: gemma3:12b で十分  
→ **複雑なバグ修正**: gemma3:27b または qwen3:8b 系を検討

### 所見4: qwen3:8b 系は type_annotate が苦手

qwen3:8b・qwen3:8b-nothink は P1 でほぼ満点だが、`04_type_annotate`（関数に型アノテーションを付加）のみ stab=0%。ファイルを書き換えるが関数本体も変えてしまうケースが観測された。

→ **型アノテーション専用タスク**: qwen3:14b か gemma3 系を使うこと。

### 所見5: phi4:14b はフェーズ0で大幅改善（6/6）、フェーズ1は不安定

システムプロンプト強化（7ルール）の効果で phi4:14b の P0 が 4/6 → **6/6** に向上。ただし P1 では T2通過ケースの多くで stab=33%（3回中1回のみ成功）と極めて不安定（04_type_annotate は stab=0%、05_commit_message のみ stab=100%）。P1 での採用は推奨しない。

### 所見6: codestral・mistral-nemo・granite3.3 は実用外

| モデル | 特徴 |
|---|---|
| codestral:22b | P0では T2=4/6・stab=50%（boundary/state が弱点）。P1では T2=1/6（commit_message のみ通過）と急落 |
| mistral-nemo:12b | P0では T2=2/6・stab=22%（type_bug のみ安定）。P1では T2=1/6（commit_message のみ通過） |
| granite3.3:8b | calls=0.0（全ケース）。ツール呼び出し自体が機能していない |

---

## 現時点での推薦構成

> モデルの追加・再評価のたびにここを更新する。実際の動作は `rust/crates/cli/src/args.rs` の `task_default_model()` が正。

ソブリンAIを今すぐ試みる場合の推薦モデル：

| 用途 | 推薦モデル |
|---|---|
| **精度重視（バグ修正・複雑タスク）** | `gemma3:27b` |
| **バランス（実務タスク全般）** | `gemma3:12b` |
| **軽量重視** | `qwen3:8b` / `qwen3:8b-nothink` |

**実証済みタスク（Tier1全モデル T2以上）:**
docstring追加・ユニットテスト生成・型アノテーション追加・コミットメッセージ生成

**適用対象:** S2〜S3 の機密性が高いコード

---

## 主な知見（ルーティング設計への示唆）

> 数値の一次情報は `eval/phase0/summary.md`・`eval/phase1/summary.md` を参照。
> ルーティング方針の全体設計は `docs/sovereign-ai.md` に記載している。

eval 実測ベースのデフォルトモデル割り当て：

| タスク | 推奨モデル | 根拠 |
|---|---|---|
| docstring・型アノテーション | gemma3:12b | P1 全冠・calls=1.8・高速 |
| テスト生成 | qwen3:14b | P1 全冠・test_generate stab=67% |
| コミットメッセージ | qwen3:8b-nothink | P1 5/6・calls=1.8・軽量 |
| バグ修正（複雑） | qwen3:8b | P0 6/6・boundary_bug を解決できる軽量モデル（qwen3:8b-nothink も同等） |
| バグ修正（重い） | gemma3:27b | P0 5/6・安定性が高い |

---

## sovereign-agent 固有の修正（claw-code との差異）

今回の評価を通じて以下の修正を sovereign に加えた：

| 修正 | 効果 |
|---|---|
| `parse_chunk` の tool_calls / done 判定順序を修正 | Ollama が done=true チャンクに tool_calls を乗せる問題を解消 |
| `ChatMessage` に `tool_calls` フィールド追加 | qwen3 系の会話コンテキスト崩壊（calls 暴走）を解消 |
| native mode フォールバック `parse_json_tool_call` 追加 | qwen2.5-coder の plain JSON 出力形式に対応 |
| システムプロンプトを7ルール構成に強化 | boundary_bug 等の「途中停止」問題を軽減 |
| XML suffix に「read-before-write」「write_file 必須」ルール追加 | gemma3 系の P1 スコア 3/6 → 6/6 に改善 |
| `sanitize_json_string` 追加（JSON 文字列内リテラル改行を除去） | gemma3:12b の P1 case01 を修正（did_edit=false→true） |
