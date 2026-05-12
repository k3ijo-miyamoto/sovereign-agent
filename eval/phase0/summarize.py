#!/usr/bin/env python3
"""Summarize eval results from multiple results_*.json files into a Markdown table."""

import argparse
import json
from pathlib import Path

EVAL_DIR = Path(__file__).parent
CASES_DIR = EVAL_DIR / "cases"

# Approximate model sizes in GB for recommendation context
MODEL_SIZES: dict[str, float] = {
    "gemma3:4b": 3.3,
    "qwen3:8b-nothink": 5.2,
    "qwen3:8b": 5.2,
    "llama3.1:8b": 4.9,
    "granite3.3:8b": 4.9,
    "qwen2.5:7b": 4.7,
    "gemma3:12b": 8.1,
    "mistral-nemo:12b": 7.1,
    "qwen2.5-coder:14b": 9.0,
    "qwen3:14b": 9.3,
    "phi4:14b": 9.1,
    "deepseek-coder-v2:16b": 8.9,
    "gemma3:27b": 17.0,
    "codestral:22b": 12.0,
    "devstral:24b": 14.0,
}


def tier_symbol(t1: bool, t2: bool, t3: bool) -> str:
    if t1 and t2 and t3:
        return "◎"
    if t1 and t2:
        return "○"
    if t1:
        return "△"
    return "✗"


def load_results(path: Path) -> dict:
    data = json.loads(path.read_text())
    by_case = {r["case"]: r for r in data["results"]}
    return {"model": data["model"], "by_case": by_case}


def model_stats(entry: dict) -> dict:
    by_case = entry["by_case"]
    valid = [r for r in by_case.values() if not r.get("error")]
    n = len(valid)
    calls = [r["tool_call_count"] for r in valid if r.get("tool_call_count") is not None]
    stab = [r["stability_rate"] for r in valid if r.get("stability_rate") is not None]
    return {
        "model": entry["model"],
        "n": n,
        "t1": sum(1 for r in valid if r["t1"]),
        "t2": sum(1 for r in valid if r["t2"]),
        "t3": sum(1 for r in valid if r["t3"]),
        "verify": sum(1 for r in valid if r.get("ran_python_verify")),
        "fix_ok": sum(1 for r in valid if r.get("fix_location_ok", True)),
        "minimal_ok": sum(1 for r in valid if r.get("minimal_edit_ok", True)),
        "avg_calls": round(sum(calls) / len(calls), 1) if calls else None,
        "avg_stability": round(sum(stab) / len(stab), 2) if stab else None,
        "by_case": by_case,
    }


def generate_criteria(case_dirs: list[str], case_metas: dict) -> list[str]:
    lines: list[str] = []
    lines.append("## 評価の観点")
    lines.append("")
    lines.append("### 目的")
    lines.append("")
    lines.append(
        "ローカル LLM（Ollama）を claw-code CLI 経由でコーディング支援に利用できるか検証する。"
        "ソブリンAI（外部クラウド API に依存しない AI 活用）の実現可能性を判断するための基礎評価。"
    )
    lines.append("")
    lines.append("### 評価指標")
    lines.append("")
    lines.append("| 指標 | 説明 | 判定方法 |")
    lines.append("| --- | --- | --- |")
    lines.append("| **T1** ツール使用 | モデルがファイルを実際に書き換えたか | `target.py` の内容変化を検出 |")
    lines.append("| **T2** 出力正解 | 修正後のコードが正しく動くか | `python3 target.py` の stdout を `expected_output.txt` と比較 |")
    lines.append("| **T3** 正常終了 | claw が exit code 0 で終わったか | プロセス終了コードを確認 |")
    lines.append("| **calls均** 効率性 | 平均ツール呼び出し回数 | `[Calling X]` の出現数をカウント |")
    lines.append("| **verify** 実行確認 | 修正後に `python3` で動作確認したか | bash ツール呼び出しで `python3` を検出 |")
    lines.append("| **誤修正** | 正しい箇所を修正したか | `expected_fix_pattern`（正規表現）が修正後ファイルに含まれるか |")
    lines.append("| **過剰修正** | 修正範囲が最小限か | diff 行数が `max_diff_lines` 以内か |")
    lines.append("| **安定性** | 複数回実行での再現性 | `--runs N` で N 回実行し T2 通過率を算出 |")
    lines.append("")
    lines.append("### テストケース")
    lines.append("")
    lines.append("| ケース | バグ種別 | 内容 | 最大 diff 行 | 修正パターン |")
    lines.append("| --- | --- | --- | :---: | --- |")
    for c in case_dirs:
        meta = case_metas[c]
        short = c.split("_", 1)[1] if "_" in c else c
        trap = "（安全トラップ）" if meta.get("safety_trap") else ""
        max_diff = meta.get("max_diff_lines", "—")
        pattern = meta.get("expected_fix_pattern", "—")
        lines.append(f"| {short}{trap} | {meta['bug_type']} | {meta['description']} | {max_diff} | `{pattern}` |")
    lines.append("")
    lines.append("### スコア記号")
    lines.append("")
    lines.append("| 記号 | 意味 |")
    lines.append("| :---: | --- |")
    lines.append("| ◎ | T1+T2+T3 全通過 |")
    lines.append("| ○ | T1+T2（修正は正しいが非ゼロ終了） |")
    lines.append("| △ | T1のみ（ファイルは書き換えたが出力が違う） |")
    lines.append("| ✗ | ファイル未書き換え（口頭説明のみ等） |")
    lines.append("| ⚠ | 安全トラップで変更量が閾値超過 |")
    lines.append("")
    lines.append("### モデル選定の理由")
    lines.append("")
    lines.append("以下の基準を満たすモデルを評価対象とした。")
    lines.append("")
    lines.append("**選定基準**")
    lines.append("")
    lines.append("1. **Ollama で入手可能** — ローカル実行の前提。`ollama pull` で取得できるモデルのみ対象")
    lines.append("2. **手元の環境で動作するサイズ** — 実際に起動・推論が完走できるモデルに限定（～27B）")
    lines.append("3. **モデルファミリーの多様性** — 特定のベンダー・アーキテクチャに偏らず比較できるよう選定")
    lines.append("4. **コード特化モデルと汎用モデルの両方を含む** — 用途に応じた使い分けの可否を判断するため")
    lines.append("")
    lines.append("**評価対象モデルと選定理由**")
    lines.append("")
    lines.append("| モデル | ファミリー | 開発元 | 選定理由 |")
    lines.append("| --- | --- | --- | --- |")
    lines.append("| qwen3:8b / 14b | Qwen3 | Alibaba | thinking モード搭載、コード性能が高いと報告されている |")
    lines.append("| qwen3:8b-nothink | Qwen3 | Alibaba | thinking オフ時の性能比較（同モデルの別設定） |")
    lines.append("| qwen2.5:7b | Qwen2.5 | Alibaba | Qwen3 との世代比較 |")
    lines.append("| qwen2.5-coder:14b | Qwen2.5-Coder | Alibaba | コーディング特化モデルの代表として選定 |")
    lines.append("| gemma3:4b / 12b / 27b | Gemma3 | Google | サイズ違いで性能スケールを検証 |")
    lines.append("| phi4:14b | Phi-4 | Microsoft | 小型ながら高性能として注目されているモデル |")
    lines.append("| codestral:22b | Codestral | Mistral AI | コーディング特化の大型モデル |")
    lines.append("| devstral:24b | Devstral | Mistral AI | エージェント用途向けとして設計されたモデル |")
    lines.append("| deepseek-coder-v2:16b | DeepSeek-Coder | DeepSeek | コーディング特化モデルとして広く使われている |")
    lines.append("| mistral-nemo:12b | Mistral-NeMo | Mistral AI / NVIDIA | 中型汎用モデルの代表として選定 |")
    lines.append("| llama3.1:8b | LLaMA 3.1 | Meta | 最も広く使われているオープンモデルの一つ |")
    lines.append("| granite3.3:8b | Granite 3.3 | IBM | エンタープライズ用途向けモデルとして選定 |")
    lines.append("")
    lines.append("**除外したモデル**")
    lines.append("")
    lines.append("- `llava`・`qwen2.5vl` — 画像入力専用（テキストコード修正タスクに不適）")
    lines.append("- `codegemma:7b` — 補完特化モデルのため、指示追従型の評価に不適")
    lines.append("")
    lines.append("### 制約・限界")
    lines.append("")
    lines.append("- Python 単一ファイル・単純バグ 6 種に限定（実際のコードベースは複数ファイル・大規模・複雑な依存関係）")
    lines.append("- 評価は `--plain-output prompt` モードで実施（VS Code extension の対話モードとは完全には一致しない）")
    lines.append("- 1回実行のため flaky なモデルを正確に評価できない（`--runs N` で繰り返し実行推奨）")
    lines.append("- モデルのバージョン・Ollama のバージョンに依存するため定期的な再評価が必要")
    lines.append("")
    lines.append("### 評価フロー概要")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    A[全15モデル] --> B[フェーズ1: 単回評価\\n--runs 1]")
    lines.append("    B --> C{T2 ≥ 5/6?}")
    lines.append("    C -->|Yes: 8モデル| D[フェーズ2: 安定性評価\\n--runs 3]")
    lines.append("    C -->|No: 7モデル| E[評価終了\\n実用困難と判定]")
    lines.append("    D --> F[ケース別安定性を集計]")
    lines.append("    F --> G[総合評価の結論]")
    lines.append("```")
    lines.append("")
    return lines


def generate_phase1(all_results: list[dict], case_dirs: list[str], case_metas: dict) -> list[str]:
    lines: list[str] = []
    stats = [model_stats(e) for e in all_results]
    ranked = sorted(stats, key=lambda s: (s["t2"], s["t3"]), reverse=True)
    n_cases = len(case_dirs)
    short_cases = [c.split("_", 1)[1] if "_" in c else c for c in case_dirs]

    lines.append("## フェーズ1: 全モデル単回評価（--runs 1）")
    lines.append("")
    lines.append(
        "全モデルを1回ずつ実行し、T2スコアで暫定ランキングを作成した。"
        "この段階ではモデルの揺らぎ（flakiness）は考慮していない。"
    )
    lines.append("")

    # ケース別スコア
    lines.append("### ケース別スコア")
    lines.append("")
    header = "| モデル | " + " | ".join(short_cases) + " | T2合計 |"
    sep = "| --- |" + " :---: |" * len(case_dirs) + " :---: |"
    lines.append(header)
    lines.append(sep)

    for entry in all_results:
        model = entry["model"]
        by_case = entry["by_case"]
        cells = []
        t2_total = 0
        for c in case_dirs:
            r = by_case.get(c)
            if r is None:
                cells.append("—")
            elif r.get("error"):
                cells.append("ERR")
            else:
                sym = tier_symbol(r["t1"], r["t2"], r["t3"])
                if case_metas[c].get("safety_trap") and not r.get("safety_ok", True):
                    sym += "⚠"
                cells.append(sym)
                if r["t2"]:
                    t2_total += 1
        lines.append(f"| {model} | " + " | ".join(cells) + f" | {t2_total}/{n_cases} |")

    lines.append("")
    lines.append("> ◎=T1+T2+T3全通過　○=T1+T2（終了コード非0）　△=T1のみ（修正失敗）　✗=ツール未使用　⚠=安全トラップ超過")
    lines.append("")

    # 暫定ランキング（フェーズ2選抜判定）
    lines.append("### 暫定ランキングとフェーズ2選抜")
    lines.append("")
    lines.append("| モデル | T2 | T3 | calls均 | フェーズ2選抜 |")
    lines.append("| --- | :---: | :---: | :---: | :---: |")

    for s in ranked:
        selected = is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))
        mark = "✅ 選抜" if selected else "—"
        calls = f"{s['avg_calls']}" if s["avg_calls"] is not None else "—"
        lines.append(
            f"| {s['model']} | {s['t2']}/{n_cases} | {s['t3']}/{n_cases} | {calls} | {mark} |"
        )

    lines.append("")
    lines.append("> **選抜基準**: T2 ≥ 5/6（6ケース中5ケース以上正解）を満たす8モデルをフェーズ2に進めた。")
    lines.append("")

    # フェーズ1 考察
    lines.append("### フェーズ1 考察")
    lines.append("")

    top = [s for s in ranked if s["n"] > 0 and s["t2"] / s["n"] >= 5/6]
    mid = [s for s in ranked if s["n"] > 0 and 2/6 <= s["t2"] / s["n"] < 5/6]
    low = [s for s in ranked if s["n"] == 0 or s["t2"] / s["n"] < 2/6]

    if top:
        lines.append(f"**上位（T2 5/6以上）:** {'、'.join(s['model'] for s in top)}")
        lines.append("")
    if mid:
        lines.append(f"**中位（T2 2〜4/6）:** {'、'.join(s['model'] for s in mid)}")
        lines.append("")
    if low:
        lines.append(f"**下位（T2 0〜1/6）:** {'、'.join(s['model'] for s in low)}")
        lines.append("")

    lines.append("**共通の弱点ケース:**")
    lines.append("")
    for c in case_dirs:
        results_for_case = [
            e["by_case"][c]
            for e in all_results
            if c in e["by_case"] and not e["by_case"][c].get("error")
        ]
        if not results_for_case:
            continue
        fail_rate = sum(1 for r in results_for_case if not r["t2"]) / len(results_for_case)
        short = c.split("_", 1)[1] if "_" in c else c
        if fail_rate >= 0.6:
            lines.append(f"- **{short}** — {round(fail_rate * 100)}% のモデルが修正失敗（{case_metas[c]['description']}）")
    lines.append("")

    xml_models = [s["model"] for s in stats if s["n"] > 0
                  and any(k in s["model"] for k in ["devstral", "gemma3", "codestral", "phi4", "deepseek"])]
    if xml_models:
        lines.append(f"**XMLモード動作（Ollamaのtools API非対応）:** {', '.join(xml_models)}")
        lines.append("")

    call_stats = [(s["model"], s["avg_calls"]) for s in stats if s["avg_calls"] is not None]
    if call_stats:
        call_stats.sort(key=lambda x: x[1])
        lines.append("**効率性（平均ツール呼び出し数）:**")
        lines.append("")
        lines.append("- 少ない（効率的）: " + "、".join(f"{m} ({c}回)" for m, c in call_stats[:3]))
        lines.append("- 多い（非効率）: " + "、".join(f"{m} ({c}回)" for m, c in reversed(call_stats[-3:])))
        lines.append("")

    return lines


def generate_phase2(all_results: list[dict], case_dirs: list[str]) -> list[str]:
    lines: list[str] = []
    short_cases = [c.split("_", 1)[1] if "_" in c else c for c in case_dirs]
    stats = [model_stats(e) for e in all_results]

    lines.append("## フェーズ2: 上位8モデル 安定性評価（--runs 3）")
    lines.append("")
    lines.append(
        "選抜した8モデルに対し、各ケースを3回ずつ実行して安定性（同じ問題を何割の確率で解けるか）を計測した。"
    )
    lines.append("")

    # ケース別安定性テーブル
    lines.append("### ケース別安定性")
    lines.append("")
    header = "| モデル | " + " | ".join(short_cases) + " | 平均安定性 |"
    sep = "| --- |" + " :---: |" * len(case_dirs) + " :---: |"
    lines.append(header)
    lines.append(sep)

    stability_entries = sorted(
        [e for e in all_results if is_stability_tested(e)],
        key=lambda e: model_stats(e)["avg_stability"] or 0,
        reverse=True,
    )
    for entry in stability_entries:
        s = model_stats(entry)
        cells = []
        for c in case_dirs:
            r = entry["by_case"].get(c)
            if r is None or r.get("error"):
                cells.append("—")
            else:
                rate = r.get("stability_rate")
                cells.append(f"{rate:.0%}" if rate is not None else "—")
        avg = f"{s['avg_stability']:.0%}" if s["avg_stability"] is not None else "—"
        lines.append(f"| {entry['model']} | " + " | ".join(cells) + f" | {avg} |")

    lines.append("")
    lines.append(
        "> **読み方**: 67% = 3回中2回成功、0% = 3回とも失敗。"
        " boundary_bug（off-by-one）でcodestral・devstralが0%となり安定性評価の重要性が浮き彫りになった。"
    )
    lines.append("")

    # 総合サマリ（上位8 + 全モデル）
    lines.append("### 総合サマリ")
    lines.append("")
    lines.append(
        "全指標を統合したサマリ。安定性は `--runs 3` で再評価した上位8モデルのみ意味を持つ"
        "（それ以外は単回実行の合否のため 0% or 100% となる）。"
    )
    lines.append("")
    lines.append("| モデル | T1 | T2 | T3 | calls均 | verify | 誤修正✗ | 過剰修正✗ | 安定性 |")
    lines.append("| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")

    for entry in sorted(all_results, key=lambda e: model_stats(e)["t2"], reverse=True):
        s = model_stats(entry)
        n = s["n"]
        calls = f"{s['avg_calls']}" if s["avg_calls"] is not None else "—"
        fix_ng = n - s["fix_ok"]
        min_ng = n - s["minimal_ok"]
        stab = f"{s['avg_stability']:.0%}" if s["avg_stability"] is not None else "—"
        lines.append(
            f"| {s['model']} | {s['t1']}/{n} | {s['t2']}/{n} | {s['t3']}/{n}"
            f" | {calls} | {s['verify']}/{n} | {fix_ng}/{n} | {min_ng}/{n} | {stab} |"
        )

    lines.append("")

    # フェーズ2 考察
    lines.append("### フェーズ2 考察")
    lines.append("")

    unstable = [s for s in stats if s["n"] > 0 and s["t2"] > s["t3"]]
    if unstable:
        lines.append("**T3不安定（修正は正しいが非ゼロ終了）:**")
        lines.append("")
        for s in sorted(unstable, key=lambda x: x["t2"] - x["t3"], reverse=True):
            lines.append(f"- {s['model']}: T2={s['t2']}/{s['n']} T3={s['t3']}/{s['n']}")
        lines.append("")

    fix_issues = [(s["model"], s["n"] - s["fix_ok"]) for s in stats
                  if is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))
                  and s["n"] - s["fix_ok"] > 0]
    if fix_issues:
        lines.append("**誤修正が疑われるケース（expected_fix_pattern 不一致）:**")
        lines.append("")
        for model, count in sorted(fix_issues, key=lambda x: -x[1]):
            lines.append(f"- {model}: {count}件")
        lines.append("")

    min_issues = [(s["model"], s["n"] - s["minimal_ok"]) for s in stats
                  if is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))
                  and s["n"] - s["minimal_ok"] > 0]
    if min_issues:
        lines.append("**過剰修正（max_diff_lines 超過）:**")
        lines.append("")
        for model, count in sorted(min_issues, key=lambda x: -x[1]):
            lines.append(f"- {model}: {count}件")
        lines.append("")

    # 推薦（フェーズ2の結果を受けて）
    lines.append("### 推薦")
    lines.append("")
    lines.append(
        "フェーズ2の安定性評価を踏まえた推薦。"
        "安定性テスト済みモデルは `avg_stability` を優先、未テストモデルは T2/T3 のみで判定。"
    )
    lines.append("")
    lines.append("| 用途 | 推薦モデル | T2 | 安定性 | サイズ | 理由 |")
    lines.append("| --- | --- | :---: | :---: | ---: | --- |")

    total = len(case_dirs)

    # stability-aware ranking: tested models sorted by (avg_stability, t2), others by (t2, t3)
    tested = sorted(
        [s for s in stats if is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))],
        key=lambda s: (s["avg_stability"] or 0, s["t2"]),
        reverse=True,
    )
    untested = sorted(
        [s for s in stats if not is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))
         and s["n"] > 0],
        key=lambda s: (s["t2"], s["t3"]),
        reverse=True,
    )

    # Best overall (from stability-tested)
    best = tested[0] if tested else None
    if best:
        size = MODEL_SIZES.get(best["model"], "?")
        stab = f"{best['avg_stability']:.0%}" if best["avg_stability"] is not None else "—"
        lines.append(
            f"| 精度・安定性最優先 | **{best['model']}** | {best['t2']}/{best['n']}"
            f" | {stab} | {size}GB | T2+安定性ともに最高 |"
        )

    # Second best (different model)
    second = next((s for s in tested if s["model"] != (best["model"] if best else "")), None)
    if second:
        size = MODEL_SIZES.get(second["model"], "?")
        stab = f"{second['avg_stability']:.0%}" if second["avg_stability"] is not None else "—"
        lines.append(
            f"| バランス重視 | **{second['model']}** | {second['t2']}/{second['n']}"
            f" | {stab} | {size}GB | 精度・安定性・サイズのバランスが良い |"
        )

    # Best lightweight (≤5.5GB, stability-tested preferred)
    light_tested = [s for s in tested if MODEL_SIZES.get(s["model"], 99) <= 5.5]
    light_untested = [s for s in untested if MODEL_SIZES.get(s["model"], 99) <= 5.5 and s["t2"] >= 3]
    light = (light_tested or light_untested)
    if light:
        s = light[0]
        size = MODEL_SIZES.get(s["model"], "?")
        stab = f"{s['avg_stability']:.0%}" if s["avg_stability"] is not None else "—"
        lines.append(
            f"| 軽量（〜5GB） | **{s['model']}** | {s['t2']}/{s['n']}"
            f" | {stab} | {size}GB | 小サイズで最高精度・安定性 |"
        )

    # Not recommended
    bad = [s for s in stats if s["n"] > 0 and s["t2"] == 0]
    if bad:
        names = "、".join(s["model"] for s in bad)
        lines.append(f"| 非推薦 | {names} | 0/{total} | — | — | T2=0、実用不可 |")

    lines.append("")
    return lines


def is_stability_tested(entry: dict) -> bool:
    """Return True if any case was run more than once (--runs > 1)."""
    return any(r.get("runs", 1) > 1 for r in entry["by_case"].values())


def generate_conclusion(all_results: list[dict], case_dirs: list[str]) -> list[str]:
    stats    = [model_stats(e) for e in all_results]
    n_cases  = len(case_dirs)
    boundary = "04_boundary_bug"

    # Stability-tested models ranked by (avg_stability desc, t2 desc)
    tested = sorted(
        [s for s in stats
         if is_stability_tested(next(e for e in all_results if e["model"] == s["model"]))],
        key=lambda s: (s["avg_stability"] or 0, s["t2"]),
        reverse=True,
    )

    def fmt_stab(s: dict) -> str:
        return f"{s['avg_stability']:.0%}" if s["avg_stability"] is not None else "—"

    def fmt_calls(s: dict) -> str:
        return f"{s['avg_calls']}" if s["avg_calls"] is not None else "—"

    lines: list[str] = []
    lines.append("## 総合評価の結論")
    lines.append("")

    # 第1推薦
    if tested:
        b = tested[0]
        size = MODEL_SIZES.get(b["model"], "?")
        lines.append(f"### 第1推薦: `{b['model']}`")
        lines.append("")
        lines.append(
            f"T2={b['t2']}/{n_cases}、安定性{fmt_stab(b)}、calls={fmt_calls(b)}（{size}GB）。"
            "安定性・精度ともにトップ。"
        )
        lines.append("")

    # 第2推薦（第1と別モデル）
    second = next((s for s in tested[1:] if s["model"] != (tested[0]["model"] if tested else "")), None)
    if second:
        size = MODEL_SIZES.get(second["model"], "?")
        lines.append(f"### 第2推薦: `{second['model']}`")
        lines.append("")
        lines.append(
            f"T2={second['t2']}/{n_cases}、安定性{fmt_stab(second)}、calls={fmt_calls(second)}（{size}GB）。"
            "精度・安定性・サイズのバランスが良い。"
        )
        lines.append("")

    # 軽量環境向け（≤5.5GB）
    light = [s for s in tested if MODEL_SIZES.get(s["model"], 99) <= 5.5]
    if light:
        ls   = light[0]
        size = MODEL_SIZES.get(ls["model"], "?")
        # Only emit a separate section if it's a different model from 第1推薦
        if not tested or ls["model"] != tested[0]["model"]:
            lines.append(f"### 軽量環境向け: `{ls['model']}`")
            lines.append("")
            lines.append(
                f"{size}GB で T2={ls['t2']}/{n_cases}、安定性{fmt_stab(ls)}。"
                "RAM が限られる環境の第一選択。"
            )
            lines.append("")
        else:
            lines.append(f"### 軽量環境向け: `{ls['model']}`（第1推薦と同モデル）")
            lines.append("")
            lines.append(
                f"{size}GB の小サイズでも精度・安定性ともに最高水準。軽量環境に最適。"
            )
            lines.append("")

    # 想定外の発見
    lines.append("### 想定外の発見")
    lines.append("")
    lines.append("| 発見 | 内容 |")
    lines.append("| --- | --- |")

    # boundary_bug failure rate
    if boundary in case_dirs:
        b_results = [
            e["by_case"][boundary]
            for e in all_results
            if boundary in e["by_case"] and not e["by_case"][boundary].get("error")
        ]
        if b_results:
            fail_rate = sum(1 for r in b_results if not r["t2"]) / len(b_results)
            solvers   = [s["model"] for s in stats if s["by_case"].get(boundary, {}).get("t2")]
            solver_str = "・".join(f"`{m}`" for m in solvers) if solvers else "なし"
            lines.append(
                f"| **boundary_bug が最難関** | 全モデル中{round(fail_rate * 100)}%が失敗。"
                f"解決できたのは {solver_str} のみ |"
            )

    # devstral observation
    devstral = next((s for s in stats if s["model"] == "devstral:24b"), None)
    if devstral and devstral["avg_stability"] is not None:
        b_ok = devstral["by_case"].get(boundary, {}).get("t2", False)
        b_note = "boundary_bug 0%" if not b_ok else f"boundary_bug 通過"
        lines.append(
            f"| **devstral:24b の過大評価** | エージェント特化を謳うが安定性{fmt_stab(devstral)}・{b_note}。"
            "単回評価では見えなかった弱さが露呈 |"
        )

    # codestral observation
    codestral = next((s for s in stats if s["model"] == "codestral:22b"), None)
    if codestral and codestral["avg_stability"] is not None:
        b_ok = codestral["by_case"].get(boundary, {}).get("t2", False)
        lines.append(
            f"| **codestral:22b も同様** | 安定性{fmt_stab(codestral)}、boundary_bug {'通過' if b_ok else '3回とも失敗（0%）'}。"
            "コード特化モデルだが難しいケースで崩れる |"
        )

    # Most efficient high-accuracy model (T2 >= n_cases-1, fewest calls, exclude editorial subjects)
    editorial = {devstral["model"] if devstral else "", codestral["model"] if codestral else ""}
    top_acc   = [s for s in tested
                 if s["t2"] >= n_cases - 1
                 and s["avg_calls"] is not None
                 and s["model"] not in editorial]
    if top_acc:
        eff = min(top_acc, key=lambda s: s["avg_calls"])
        lines.append(
            f"| **{eff['model']} の効率** | calls={fmt_calls(eff)}は読み込み・修正・確認の最小手数。無駄なループをしない |"
        )

    lines.append("")

    # ソブリンAI実用観点での結論
    # 実用レベル: T2 >= n_cases-1 かつ stab >= 75% かつ editorial 対象外
    practical  = [s["model"] for s in tested
                  if s["t2"] >= n_cases - 1
                  and (s["avg_stability"] or 0) >= 0.75
                  and s["model"] not in editorial]
    # 軽量（≤5.5GB）かつ実用レベル外だが T2 > n_cases//2 のモデル
    light_only = [s["model"] for s in light
                  if s["model"] not in practical and s["t2"] > n_cases // 2]
    lines.append("### ソブリンAI実用観点での結論")
    lines.append("")
    lines.append("「社内の機密コードを外部に送らずに自動修正したい」という用途において:")
    lines.append("")
    if practical:
        lines.append(f"- **現時点で実用レベル**: {'、'.join(f'`{m}`' for m in practical)}")
    if light_only:
        lines.append(f"- **軽量で十分**: {'、'.join(f'`{m}`' for m in light_only)}（小規模・定型タスク）")
    if devstral:
        lines.append("- **期待を下回った**: `devstral:24b`（エージェント特化を謳うが安定性不足）")
    lines.append("")
    lines.append(
        "> **注意**: 本評価は単一ファイル・単純バグ6種に限定。"
        "実際の複数ファイル・大規模コードベースでは別途検証が必要。"
    )
    lines.append("")
    return lines


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize claw-code eval results")
    parser.add_argument(
        "--output", "-o", help="Write Markdown to this file (default: stdout)"
    )
    parser.add_argument(
        "--results-dir",
        default=str(EVAL_DIR / "results"),
        help="Directory containing *.json result files",
    )
    args = parser.parse_args()

    results_dir = Path(args.results_dir)
    result_files = sorted(results_dir.glob("*.json"))
    if not result_files:
        print("No *.json files found.")
        return

    all_results = [load_results(f) for f in result_files]

    case_dirs = sorted(d.name for d in CASES_DIR.iterdir() if d.is_dir())
    case_dirs = [c for c in case_dirs if (CASES_DIR / c / "meta.json").exists()]

    case_metas = {}
    for c in case_dirs:
        meta = json.loads((CASES_DIR / c / "meta.json").read_text())
        case_metas[c] = meta

    lines: list[str] = []

    lines += generate_criteria(case_dirs, case_metas)
    lines += generate_phase1(all_results, case_dirs, case_metas)
    lines += generate_phase2(all_results, case_dirs)
    lines += generate_conclusion(all_results, case_dirs)

    output = "\n".join(lines)

    if args.output:
        Path(args.output).write_text(output)
        print(f"Written to {args.output}")
    else:
        print(output)


if __name__ == "__main__":
    main()
