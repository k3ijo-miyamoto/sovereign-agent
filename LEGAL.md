# 法律的検討メモ

このドキュメントは、sovereign-agent が著作権上問題のない独立した実装である
ことを確認した検討プロセスを記録したものです。

---

## 作成の経緯

前身プロジェクト claw-code には以下の著作権連鎖があった：

```
Anthropic 誤公開コード → Yeachan-Heo Python ポート → claw-code
```

claw-code の一部（rust/, src/）は上記の派生物である可能性があり、
そのまま公開するとリスクがあると判断した。

そのため、問題のあるコードを**一切参照せずに**クリーンルーム実装として
sovereign-agent を作成した。

---

## クリーンルームと判断した根拠

### 1. コードの独立性

sovereign-agent の全 Rust コードは、claw-code/rust/ および claw-code/src/ を
参照せずに新規に書いた。

実装の独立性を示す具体的な差異：

| 機能 | claw-code/rust | sovereign-agent/rust |
|---|---|---|
| bash 実行 | 283行（サンドボックス・namespace隔離・バックグラウンド実行） | 29行（`bash -c cmd` のシンプルな実行） |
| SSE パーサー | 128行（RFC準拠のステートマシン型増分パーサー） | 12行（`"data: "` を剥がすだけ） |
| ツール数 | bash/glob/grep/read/write/edit 等 多数 | bash/read_file/write_file/list_files の4つ |
| 追加機能 | MCP・OAuth・サンドボックス・セッション永続化・コンパクション | なし |

構造・規模・実装方針のいずれも別物であり、**実質的類似性（substantial similarity）はない**。

### 2. 参照した情報源

実装にあたって参照したのは以下の公開情報のみ：

- Ollama API ドキュメント（`/api/chat` エンドポイントの仕様）
- Anthropic Messages API ドキュメント（公開されている API リファレンス）
- VS Code Extension API ドキュメント
- Rust 標準ライブラリおよび各クレートのドキュメント

これらはすべてパブリックドメインまたは公開仕様であり、
これらを参照して実装を行うことに法的問題はない。

### 3. 依存クレートのライセンス確認

`cargo deny check licenses` による機械的確認を実施：

```
licenses ok
```

使用している全クレートのライセンス：

| ライセンス | 代表的なクレート |
|---|---|
| MIT または Apache-2.0 | tokio, reqwest, serde, anyhow, futures 等 |
| Apache-2.0 WITH LLVM-exception | 一部コンパイラ関連クレート |
| BSD-3-Clause | encoding_rs 等 |
| Unicode-3.0 | icu 系クレート |

すべて OSI 承認済みのオープンソースライセンスであり、MIT との互換性がある。

### 4. VS Code 拡張について

VS Code 拡張のコードはユーザーが独自に実装したもの。
claw-code の VS Code 拡張もユーザー自身が書いたものであるため、
そこから**着想を得ること**は著作権上の問題にはならない
（著作権は表現を保護するものであり、アイデア・着想を保護するものではない）。

---

## 結論

| 観点 | 判断 | 根拠 |
|---|---|---|
| Rust コードの独立性 | ✅ 問題なし | ゼロから実装、実質的類似性なし |
| VS Code 拡張 | ✅ 問題なし | ユーザー独自実装 |
| 依存ライセンス | ✅ 問題なし | cargo deny で全件確認 |
| 参照情報源 | ✅ 問題なし | 公開 API ドキュメントのみ |

**sovereign-agent は GitHub への公開に法律的問題はないと判断する。**

---

## ライセンス

本プロジェクトは MIT License のもとで公開する（[LICENSE](./LICENSE) 参照）。

---

*このメモは法的アドバイスではありません。正式な判断が必要な場合は弁護士に相談してください。*
