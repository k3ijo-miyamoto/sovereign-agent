# フェーズ0 + フェーズ1 評価サマリ（sovereign-agent・全15モデル）

> 更新日: 2026-05-11
> ハーネス: `eval/run_eval.py`（Phase 0）/ `eval/run_eval_phase1.py`（Phase 1）
> バイナリ: `rust/target/release/sovereign`

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

「nth_fibonacci が range(n) を使っておりオフバイワン」という境界値バグは、多くのモデルが「出力は正しく見える」罠にはまって失敗する。解決できたのは **qwen3:8b・qwen3:8b-nothink・phi4:14b** の3モデルのみ。

→ 複雑なロジックバグは qwen3:8b 系が最も信頼できる。

### 所見3: コスト最適解は gemma3:12b（フェーズ1限定）

フェーズ1（docstring・テスト生成・型アノテーション・コミットメッセージ）では、gemma3:12b が 6/6 かつ calls=1.8（最軽量クラス）を達成。モデルサイズが27b の半分でほぼ同等の成果を出している。

→ **実務タスク全般**: gemma3:12b で十分  
→ **複雑なバグ修正**: gemma3:27b または qwen3:8b 系を検討

### 所見4: qwen3:8b 系は type_annotate が苦手

qwen3:8b・qwen3:8b-nothink は P1 でほぼ満点だが、`04_type_annotate`（関数に型アノテーションを付加）のみ stab=0%。ファイルを書き換えるが関数本体も変えてしまうケースが観測された。

→ **型アノテーション専用タスク**: qwen3:14b か gemma3 系を使うこと。

### 所見5: phi4:14b はフェーズ0で大幅改善（6/6）、フェーズ1は不安定

システムプロンプト強化（7ルール）の効果で phi4:14b の P0 が 4/6 → **6/6** に向上。ただし P1 では stab が全ケースで 33%（3回中1回しか成功しない）と極めて不安定。P1 での採用は推奨しない。

### 所見6: codestral・mistral-nemo・granite3.3 は実用外

| モデル | 特徴 |
|---|---|
| codestral:22b | calls=0 のケースが多く、ツールを呼ばずに終了するパターンが支配的 |
| mistral-nemo:12b | 常に calls=1.0（read のみ）。write_file を決して呼ばない |
| granite3.3:8b | calls=0.0（全ケース）。ツール呼び出し自体が機能していない |

---

## フェーズ0（バグ修正）結果

### 集計表

| モデル | T2 | stab | boundary_bug | 特記 |
|---|---|---|---|---|
| qwen3:8b-nothink | **6/6** | 89% | ✅ | 唯一の全冠・高安定 |
| qwen3:8b | **6/6** | 81% | ✅ | 全冠・boundary_bug stab=33% |
| phi4:14b | **6/6** | 78% | ✅ | 全冠・stab やや低め |
| gemma3:27b | 5/6 | 83% | ❌ | boundary のみ stab=0% |
| gemma3:12b | 5/6 | 78% | ❌ | boundary のみ stab=0% |
| qwen3:14b | 5/6 | 83% | ❌ | boundary のみ stab=0% |
| qwen2.5-coder:14b | 5/6 | 83% | ❌ | boundary のみ stab=0% |
| qwen2.5:7b | 5/6 | 72% | ❌ | boundary のみ stab=0% |
| devstral:24b | 5/6 | 78% | ❌ | boundary のみ stab=0% |
| deepseek-coder-v2:16b | 5/6 | 61% | ✅ | syntax_bug が stab=0% |
| codestral:22b | 2/6 | 11% | ❌ | ツール不呼出が多発 |
| llama3.1:8b | 3/6 | 17% | ❌ | calls 暴走（avg 13〜19） |
| gemma3:4b | 1/6 | 6% | ❌ | 実用不可 |
| mistral-nemo:12b | 0/6 | 0% | ❌ | 実用不可 |
| granite3.3:8b | 0/6 | 0% | ❌ | 実用不可 |

### ケース別詳細（上位6モデル）

| ケース | g27b | g12b | q3:14b | q3:8b | q3:8b-nt | q2.5c:14b |
|---|---|---|---|---|---|---|
| 01_syntax_bug | ✅100% | ✅67% | ✅100% | ✅50% | ✅100% | ✅100% |
| 02_type_bug | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |
| 03_logic_bug | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |
| 04_boundary_bug | ❌0% | ❌0% | ❌0% | ✅33% | ✅33% | ❌0% |
| 05_state_bug | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |
| 06_safety_trap | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |

---

## フェーズ1（実務タスク）結果

### 集計表

| モデル | T2 | avg calls | stab(avg) | 特記 |
|---|---|---|---|---|
| gemma3:27b | **6/6** | 2.1 | 94% | 全ケース安定 |
| gemma3:12b | **6/6** | 1.8 | 89% | 最効率・最コスパ |
| qwen3:14b | **6/6** | 1.9 | 78% | 全冠・02 complex が stab 低め |
| qwen2.5-coder:14b | **6/6** | 2.7 | 78% | 全冠・calls やや多め |
| qwen3:8b | 5/6 | 1.9 | 72% | 04_type_annotate のみ ❌ |
| qwen3:8b-nothink | 5/6 | 1.8 | 78% | 04_type_annotate のみ ❌ |
| phi4:14b | 5/6 | 1.8 | 39% | 5/6 だが全ケースで stab=33% |
| devstral:24b | 5/6 | 1.7 | 56% | 05_commit_message のみ ❌ |
| deepseek-coder-v2:16b | 3/6 | 1.3 | 39% | 02・03 が苦手 |
| qwen2.5:7b | 2/6 | 2.1 | 33% | docstring 全滅 |
| codestral:22b | 1/6 | 1.0 | 6% | 03_test_generate のみ通過 |
| mistral-nemo:12b | 1/6 | 1.1 | 6% | 05 のみ通過 |
| gemma3:4b | 1/6 | 1.9 | 6% | 05 のみ通過 |
| llama3.1:8b | 1/6 | 4.1 | 6% | calls 暴走 |
| granite3.3:8b | 0/6 | 0.0 | 0% | 実用不可 |

### ケース別詳細（上位4モデル）

| ケース | g27b | g12b | q3:14b | q2.5c:14b |
|---|---|---|---|---|
| 01_docstring_generate | ✅67% | ✅100% | ✅100% | ✅100% |
| 02_docstring_complex | ✅100% | ✅100% | ✅33% | ✅33% |
| 02b_docstring_complex | ✅100% | ✅67% | ✅67% | ✅67% |
| 03_test_generate | ✅100% | ✅67% | ✅67% | ✅67% |
| 04_type_annotate | ✅100% | ✅100% | ✅100% | ✅100% |
| 05_commit_message | ✅100% | ✅100% | ✅100% | ✅100% |

---

## 主な知見（ルーティング設計への示唆）

CLAUDE.md の自動ルーティング設計方針（Phase A）に対応するデフォルト値：

| タスク | 推奨モデル | 根拠 |
|---|---|---|
| docstring・型アノテーション | gemma3:12b | P1 全冠・calls=1.8・高速 |
| テスト生成 | qwen3:14b | P1 全冠・test_generate stab=67% |
| コミットメッセージ | qwen3:8b-nothink | P1 5/6・calls=1.8・軽量 |
| バグ修正（複雑） | qwen3:8b | P0 6/6・boundary_bug を解決できる唯一の軽量モデル |
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
