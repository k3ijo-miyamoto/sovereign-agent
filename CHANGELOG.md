# CHANGELOG — sovereign-agent 固有の修正

eval 実施を通じて sovereign 本体に加えた修正の記録。

## 2026-05（Phase 0 / Phase 1 評価）

| 修正 | 効果 |
|---|---|
| `parse_chunk` の tool_calls / done 判定順序を修正 | Ollama が done=true チャンクに tool_calls を乗せる問題を解消 |
| `ChatMessage` に `tool_calls` フィールド追加 | qwen3 系の会話コンテキスト崩壊（calls 暴走）を解消 |
| native mode フォールバック `parse_json_tool_call` 追加 | qwen2.5-coder の plain JSON 出力形式に対応 |
| システムプロンプトを7ルール構成に強化 | boundary_bug 等の「途中停止」問題を軽減 |
| XML suffix に「read-before-write」「write_file 必須」ルール追加 | gemma3 系の P1 スコア 3/6 → 6/6 に改善 |
| `sanitize_json_string` 追加（JSON 文字列内リテラル改行を除去） | gemma3:12b の P1 case01 を修正（did_edit=false→true） |
