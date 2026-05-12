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
| bash 実行 | 283行（サンドボックス・namespace隔離・バックグラウンド実行） | 57行（`bash -c cmd` のシンプルな実行） |
| SSE パーサー | 128行（RFC準拠のステートマシン型増分パーサー） | 12行（`"data: "` を剥がすだけ） |
| ツール数 | bash/glob/grep/read/write/edit 等 多数 + MCP・OAuth 等 | bash/read_file/write_file/list_files/grep_search/glob_search/edit_file の7つ |
| MCP | フル実装（OAuth・プロセス管理・セッション永続化） | 最小実装（stdio プロセス起動・ツール一覧取得・呼び出しのみ） |
| その他 | OAuth・サンドボックス・セッション永続化・コンパクション | なし |

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

---

# Legal Review Notes (English)

This document records the review process confirming that sovereign-agent is an independently implemented project free of copyright concerns.

---

## Background

The predecessor project claw-code had the following copyright chain:

```
Anthropic accidentally-leaked code → Yeachan-Heo Python port → claw-code
```

Parts of claw-code (rust/, src/) may be derivatives of the above, and publishing them as-is was judged to carry legal risk.

Therefore, sovereign-agent was created as a clean-room implementation **without referencing any of the problematic code**.

---

## Basis for Clean-Room Determination

### 1. Code Independence

All Rust code in sovereign-agent was written from scratch without referencing claw-code/rust/ or claw-code/src/.

Concrete differences demonstrating implementation independence:

| Feature | claw-code/rust | sovereign-agent/rust |
|---|---|---|
| bash execution | 283 lines (sandbox, namespace isolation, background execution) | 57 lines (simple `bash -c cmd` execution) |
| SSE parser | 128 lines (RFC-compliant stateful incremental parser) | 12 lines (strips `"data: "` prefix only) |
| Tool count | bash/glob/grep/read/write/edit + MCP/OAuth etc. | bash/read_file/write_file/list_files/grep_search/glob_search/edit_file (7 tools) |
| MCP | Full implementation (OAuth, process management, session persistence) | Minimal implementation (stdio process launch, tool listing, invocation only) |
| Other | OAuth, sandbox, session persistence, compaction | None |

The structure, scale, and implementation approach are all distinct — **there is no substantial similarity**.

### 2. References Used

The only sources referenced during implementation were publicly available:

- Ollama API documentation (specification for the `/api/chat` endpoint)
- Anthropic Messages API documentation (public API reference)
- VS Code Extension API documentation
- Rust standard library and crate documentation

All of these are in the public domain or are published specifications; referencing them raises no legal concerns.

### 3. Dependency License Verification

Mechanical verification was performed using `cargo deny check licenses`:

```
licenses ok
```

Licenses of all crates used:

| License | Representative crates |
|---|---|
| MIT or Apache-2.0 | tokio, reqwest, serde, anyhow, futures, etc. |
| Apache-2.0 WITH LLVM-exception | Some compiler-related crates |
| BSD-3-Clause | encoding_rs, etc. |
| Unicode-3.0 | icu-related crates |

All are OSI-approved open-source licenses compatible with MIT.

### 4. VS Code Extension

The VS Code extension code was independently implemented by the user.
Because the claw-code VS Code extension was also written by the same user,
**drawing inspiration** from it raises no copyright concern
(copyright protects expression, not ideas or concepts).

---

## Conclusion

| Aspect | Judgment | Basis |
|---|---|---|
| Rust code independence | ✅ No issue | Implemented from scratch; no substantial similarity |
| VS Code extension | ✅ No issue | User's own independent implementation |
| Dependency licenses | ✅ No issue | All verified via cargo deny |
| References used | ✅ No issue | Public API documentation only |

**sovereign-agent is determined to have no legal issues with publication on GitHub.**

---

## License

This project is published under the MIT License (see [LICENSE](./LICENSE)).

---

*This note does not constitute legal advice. Consult a qualified attorney for formal legal opinions.*
