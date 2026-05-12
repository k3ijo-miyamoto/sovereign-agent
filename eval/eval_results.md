# フェーズ0 + フェーズ1 評価サマリ（sovereign-agent・全15モデル）

> 更新日: 2026-05-12
> ハーネス: `eval/phase0/run_eval.py`（Phase 0）/ `eval/phase1/run_eval.py`（Phase 1）
> バイナリ: `rust/target/debug/sovereign` (git: 38d26e4)
> 実行条件: --runs 3（全モデル）、ollama 0.23.1

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

### 所見1: 実用に耐えるモデルは7つに絞られる

両フェーズで安定して動くのは以下7モデル。

| モデル | P0 T2 | P0 stab | P1 T2 | P1 stab | 特記 |
|---|---|---|---|---|---|
| **qwen3:14b** | 6/6 | 100% | 6/6 | 83% | P0 完全制覇・安定性100%・P1 全冠 |
| **gemma3:12b** | 5/6 | 83% | 6/6 | 100% | P1 全冠・stab 100%・コスト優位 |
| **gemma3:27b** | 5/6 | 78% | 6/6 | 89% | P1 全冠・高安定性 |
| **qwen3:8b** | 6/6 | 86% | 6/6 | 75% | P0・P1 ともに全冠・軽量最強 |
| **qwen2.5-coder:14b** | 5/6 | 67% | 6/6 | 72% | P1 全冠 |
| **devstral:24b** | 5/6 | 72% | 6/6 | 61% | P1 全冠（新発見） |
| **phi4:14b** | 6/6 | 89% | 6/6 | 50% | P0 高精度・P1 は不安定 |

### 所見2: boundary_bug はほぼ全モデルの壁

「nth_fibonacci が range(n) を使っておりオフバイワン」という境界値バグは、多くのモデルが「出力は正しく見える」罠にはまって失敗する。解決できたのは **qwen3:14b（stab=100%）・qwen3:8b（67%）・phi4:14b（33%）・deepseek-coder-v2:16b（33%）** の4モデルのみ。

→ 複雑なロジックバグは qwen3:14b が最も信頼できる（stab=100%）。

### 所見3: コスト最適解は gemma3:12b（フェーズ1）

フェーズ1（docstring・テスト生成・型アノテーション・コミットメッセージ）では、gemma3:12b が **T2=6/6・stab=100%・calls=2.1** を達成。27b の半分のサイズで完全同等以上の成果。

→ **実務タスク全般**: gemma3:12b で十分  
→ **複雑なバグ修正**: qwen3:14b または qwen3:8b を検討

### 所見4: qwen3:8b は P0・P1 ともに全冠

qwen3:8b は今回の評価で P0 6/6・P1 6/6 を達成（P1 type_annotate も stab=50% で通過）。qwen3:8b-nothink は type_annotate のみ stab=0% で失敗している。

→ **型アノテーションを含むタスク**: qwen3:8b（nothink でない）か gemma3 系を使うこと。

### 所見5: devstral:24b は P1 で全冠（新発見）

今回の評価で devstral:24b が P1 T2=6/6 を達成。ただし stab=61% と安定性は低め（特に docstring 系は stab=33%）。P0 では boundary_bug が 0% で安定性は 72%。

→ Mistral のエージェント特化設計が P1 実務タスクで活きているが、flaky なため本番採用は要注意。

### 所見6: phi4:14b は P0 で高精度（6/6）、P1 は不安定（stab=50%）

P0 では T2=6/6・stab=89%・boundary_bug も 33% と上位に入るが、P1 では stab=50%（各ケースで 33-67%）と極めて不安定。

### 所見7: codestral・mistral-nemo・granite3.3 は実用外

| モデル | 特徴 |
|---|---|
| codestral:22b | P0 T2=4/6・stab=50%（boundary/state が弱点）。P1 T2=2/6（commit_message のみ安定） |
| mistral-nemo:12b | P0 T2=4/6・stab=22%。P1 T2=1/6（commit_message のみ通過） |
| granite3.3:8b | calls=0.0（全ケース）。ツール呼び出し自体が機能していない |

---

## 現時点での推薦構成

> モデルの追加・再評価のたびにここを更新する。実際の動作は `rust/crates/cli/src/args.rs` の `task_default_model()` が正。

| 用途 | 推薦モデル | P0 stab | P1 stab | 根拠 |
|---|---|---|---|---|
| **精度重視（バグ修正・複雑タスク）** | `qwen3:14b` | 100% | 83% | P0・P1 全冠、boundary_bug stab=100% |
| **バランス（実務タスク全般）** | `gemma3:12b` | 83% | 100% | P1 全冠・stab100%・コスト優位 |
| **軽量重視（〜5GB）** | `qwen3:8b` | 86% | 75% | P0・P1 全冠・最軽量クラス |

**実証済みタスク（Tier1全モデル T2以上）:**
docstring追加・ユニットテスト生成・型アノテーション追加・コミットメッセージ生成・バグ修正（単純〜複雑）

**適用対象:** S2〜S3 の機密性が高いコード

---

## 主な知見（ルーティング設計への示唆）

> 数値の一次情報は `eval/phase0/summary.md`・`eval/phase1/summary.md` を参照。
> ルーティング方針の全体設計は `docs/sovereign-ai.md` に記載している。

eval 実測ベースのデフォルトモデル割り当て：

| タスク | 推薦モデル | 根拠 |
|---|---|---|
| docstring・型アノテーション | gemma3:12b | P1 全冠・stab=100%・calls=2.1 |
| テスト生成 | gemma3:12b | P1 全冠・stab=100%・covers_partial 唯一通過 |
| コミットメッセージ | qwen3:8b-nothink | P1 commit_message stab=100%・calls=1.8・軽量 |
| バグ修正（複雑） | qwen3:14b | P0 6/6・boundary_bug stab=100%・安定性最高 |
| バグ修正（軽量） | qwen3:8b | P0 6/6 stab=86%・P1 全冠・最軽量クラス |

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
