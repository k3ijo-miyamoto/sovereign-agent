# Phase 1 評価サマリ（実務タスク・全モデル）

> 更新日: 2026-05-12
> ハーネス: `eval/phase1/run_eval.py`
> ケース数: 6（docstring_generate, 02_docstring_complex, 02b_docstring_complex, test_generate, type_annotate, commit_message）

---

## 集計表

| モデル | T2 | T3 | avg calls | stab(avg) | 特記 |
|---|:---:|:---:|:---:|:---:|---|
| gemma3:12b | **6/6** | 5/6 | 2.1 | 100% | 全冠 |
| gemma3:27b | **6/6** | 5/6 | 2.2 | 89% | 全冠 |
| qwen3:14b | **6/6** | 5/6 | 2.1 | 83% | 全冠 |
| qwen3:8b | **6/6** | 5/6 | 1.9 | 75% | 全冠 |
| qwen2.5-coder:14b | **6/6** | 4/6 | 2.7 | 72% | 全冠 |
| devstral:24b | **6/6** | 5/6 | 1.9 | 61% | 全冠 |
| phi4:14b | **6/6** | 5/6 | 2.3 | 50% | 全冠 |
| qwen3:8b-nothink | **5/6** | 4/6 | 1.8 | 72% |  |
| qwen2.5:7b | **4/6** | 3/6 | 3.3 | 39% |  |
| deepseek-coder-v2:16b | **3/6** | 1/6 | 1.7 | 33% |  |
| codestral:22b | **2/6** | 0/6 | 2.4 | 11% |  |
| gemma3:4b | **1/6** | 1/6 | 3.9 | 17% |  |
| mistral-nemo:12b | **1/6** | 1/6 | 2.5 | 17% |  |
| llama3.1:8b | **1/6** | 1/6 | 2.7 | 6% |  |
| granite3.3:8b | **0/6** | 0/6 | 0.0 | 0% | 実用不可 |

## ケース別 T2 結果

| モデル | docstring_generate | 02_docstring_complex | 02b_docstring_complex | test_generate | type_annotate | commit_message |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| devstral:24b | ✅33% | ✅33% | ✅67% | ✅67% | ✅67% | ✅100% |
| gemma3:12b | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |
| gemma3:27b | ✅33% | ✅100% | ✅100% | ✅100% | ✅100% | ✅100% |
| phi4:14b | ✅33% | ✅33% | ✅67% | ✅33% | ✅33% | ✅100% |
| qwen2.5-coder:14b | ✅33% | ✅33% | ✅67% | ✅100% | ✅100% | ✅100% |
| qwen3:14b | ✅100% | ✅67% | ✅67% | ✅67% | ✅100% | ✅100% |
| qwen3:8b | ✅100% | ✅33% | ✅67% | ✅100% | ✅50% | ✅100% |
| qwen3:8b-nothink | ✅67% | ✅67% | ✅100% | ✅100% | ❌ | ✅100% |
| qwen2.5:7b | ❌ | ✅33% | ❌ | ✅33% | ✅67% | ✅100% |
| deepseek-coder-v2:16b | ✅33% | ❌ | ❌ | ❌ | ✅67% | ✅100% |
| codestral:22b | ✅33% | ❌ | ❌ | ❌ | ❌ | ✅33% |
| gemma3:4b | ❌ | ❌ | ❌ | ❌ | ❌ | ✅100% |
| llama3.1:8b | ❌ | ❌ | ❌ | ❌ | ❌ | ✅33% |
| mistral-nemo:12b | ❌ | ❌ | ❌ | ❌ | ❌ | ✅100% |
| granite3.3:8b | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

> ✅ = T2通過（安定性）、❌ = T2失敗

## ケース別 checks 詳細（上位4モデル）

### docstring_generate

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `no_body_change` | 関数本体を変更していない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `docstring_added` | docstringが追加されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_empty` | 空リストの挙動に触れている |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_same_value` | 全要素同一の場合（0.0返却）に触れている |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_normalization` | 0〜1正規化であることを説明している | ✓ | ✅ | ✅ | ✅ | ✅ |
| `syntax_valid` | Pythonとして構文エラーがない | ✓ | ✅ | ✅ | ✅ | ✅ |

### 02_docstring_complex

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `syntax_valid` | Pythonとして構文エラーがない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `docstring_added` | 関数にdocstringが追加されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `no_body_change` | 関数本体を変更していない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_weights` | 重み付き平均（weightsパラメータ）への言及がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_pad_value` | min_periods未満の場合のpad_valueへの言及がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_min_periods` | min_periodsの挙動（デフォルト=window）への言及がある |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_empty` | 空リスト入力への言及がある |  | ❌ | ❌ | ❌ | ❌ |
| `mentions_weights_alignment` | weightsの右寄せアライメント（window未満時）への言及がある |  | ❌ | ❌ | ❌ | ❌ |

### 02b_docstring_complex

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `syntax_valid` | Pythonとして構文エラーがない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `docstring_added` | 関数にdocstringが追加されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `no_body_change` | 関数本体を変更していない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_weights` | 重み付き平均（weightsパラメータ）への言及がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_pad_value` | min_periods未満の場合のpad_valueへの言及がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_min_periods` | min_periodsの挙動（デフォルト=window）への言及がある |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_empty` | 空リスト入力への言及がある |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_weights_alignment` | weightsの右寄せアライメント（window未満時）への言及がある |  | ❌ | ❌ | ❌ | ❌ |

### test_generate

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `test_file_created` | test_target.pyが作成されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `target_unchanged` | target.pyを変更していない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `pytest_pass` | pytest test_target.py が全テスト通過（exit 0） | ✓ | ✅ | ✅ | ✅ | ✅ |
| `covers_error` | size<=0のValueErrorテストがある |  | ✅ | ✅ | ✅ | ✅ |
| `covers_empty` | 空リスト入力のテストがある |  | ✅ | ✅ | ✅ | ✅ |
| `covers_partial` | 末尾の余りchunkのテストがある |  | ❌ | ✅ | ❌ | ❌ |

### type_annotate

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `syntax_valid` | Pythonとして構文エラーがない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `annotations_added` | 少なくとも1つの型アノテーションが追加されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `no_body_change` | 関数本体を変更していない | ✓ | ✅ | ✅ | ✅ | ✅ |
| `has_return_type` | 戻り値の型アノテーション（->）がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `all_annotated` | 全関数の全引数と戻り値に型アノテーションが付いている |  | ✅ | ✅ | ✅ | ✅ |

### commit_message

| check | 説明 | T2必須 | devstral:24b | gemma3:12b | gemma3:27b | phi4:14b |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `file_created` | commit_message.txt が生成されている | ✓ | ✅ | ✅ | ✅ | ✅ |
| `has_verb` | 変更の種類を示す動詞がある | ✓ | ✅ | ✅ | ✅ | ✅ |
| `mentions_change` | 変更対象（weighted average / 関数名）に言及している | ✓ | ✅ | ✅ | ✅ | ✅ |
| `subject_short` | subject行が72文字以内 |  | ✅ | ✅ | ✅ | ✅ |
| `mentions_parameter` | weightsパラメータについて言及している |  | ✅ | ✅ | ✅ | ✅ |
