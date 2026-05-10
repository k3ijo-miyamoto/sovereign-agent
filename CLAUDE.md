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

## XMLモード

以下のモデルプレフィックスは Ollama の native tools API 非対応のため XML モードで動作する:
`gemma3`, `phi4`, `codestral`, `devstral`, `deepseek`

XMLモードではツール定義をシステムプロンプトに埋め込み、
モデルは `<tool_call>{"name":"...","arguments":{...}}</tool_call>` で応答する。

## evalハーネス

`eval/` は claw-code から移管したオリジナルのローカルLLM評価基盤。
使い方は eval/ 内のスクリプトを参照。
sovereign バイナリへの接続は今後実装予定。
