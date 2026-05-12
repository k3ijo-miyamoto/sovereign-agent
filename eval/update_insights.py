#!/usr/bin/env python3
"""README.md の insights セクションを eval JSON 結果から自動生成する。"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
P0_DIR = ROOT / "eval" / "phase0" / "results"
P1_DIR = ROOT / "eval" / "phase1" / "results"
README = ROOT / "README.md"

MODEL_SIZE = {
    "gemma3:4b":              "3.3GB",
    "gemma3:12b":             "8.1GB",
    "gemma3:27b":             "17.0GB",
    "phi4:14b":               "9.1GB",
    "devstral:24b":           "14.0GB",
    "qwen3:8b":               "5.2GB",
    "qwen3:8b-nothink":       "5.2GB",
    "qwen3:14b":              "9.3GB",
    "qwen2.5:7b":             "4.7GB",
    "qwen2.5-coder:14b":      "9.0GB",
    "codestral:22b":          "12.9GB",
    "deepseek-coder-v2:16b":  "8.9GB",
    "mistral-nemo:12b":       "7.1GB",
    "llama3.1:8b":            "4.9GB",
    "granite3.3:8b":          "5.1GB",
}


def load_phase(results_dir: Path) -> dict[str, dict]:
    data = {}
    for f in results_dir.glob("*.json"):
        d = json.loads(f.read_text())
        model = d["model"]
        results = d.get("results", [])
        t2 = sum(1 for r in results if r.get("t2"))
        stabs = [r["stability_rate"] for r in results if r.get("stability_rate") is not None]
        avg_stab = sum(stabs) / len(stabs) if stabs else 0.0
        calls_ok = [r.get("tool_call_count", 0) for r in results if r.get("t2")]
        avg_calls = sum(calls_ok) / len(calls_ok) if calls_ok else 0.0
        data[model] = {
            "t2": t2, "n": len(results),
            "stab": avg_stab, "calls": avg_calls,
            "by_case": {r["case"]: r for r in results},
        }
    return data


def pct(v: float) -> str:
    return f"{round(v * 100)}%"


def generate_insights(p0: dict, p1: dict) -> list[str]:
    bullets = []

    # 1. P0 総合首位
    p0_top = sorted(p0.items(), key=lambda x: (x[1]["t2"], x[1]["stab"]), reverse=True)[0]
    m, s = p0_top
    size = MODEL_SIZE.get(m, "")
    p1s = p1.get(m, {})
    bullets.append(
        f"**{m} が総合首位** — "
        f"P0 T2={s['t2']}/{s['n']} stab={pct(s['stab'])}・"
        f"P1 T2={p1s.get('t2', '?')}/6 stab={pct(p1s.get('stab', 0))}（{size}）"
    )

    # 2. P1 実務タスク最適（stab 最高）
    p1_top = sorted(p1.items(), key=lambda x: (x[1]["t2"], x[1]["stab"]), reverse=True)[0]
    m1, s1 = p1_top
    size1 = MODEL_SIZE.get(m1, "")
    bullets.append(
        f"**{m1} は実務タスクに最適** — "
        f"P1 T2={s1['t2']}/6 stab={pct(s1['stab'])}・calls={s1['calls']:.1f}（{size1}）"
    )

    # 3. 5GB 以下の軽量モデル
    light = {m: s for m, s in p0.items() if MODEL_SIZE.get(m, "99GB") <= "5.9GB"}
    if light:
        best_light = sorted(light.items(), key=lambda x: (x[1]["t2"], x[1]["stab"]), reverse=True)[0]
        ml, sl = best_light
        p1l = p1.get(ml, {})
        bullets.append(
            f"**{ml} は軽量最強（{MODEL_SIZE.get(ml, '')}）** — "
            f"P0 T2={sl['t2']}/{sl['n']} stab={pct(sl['stab'])}・"
            f"P1 T2={p1l.get('t2', '?')}/6 stab={pct(p1l.get('stab', 0))}"
        )

    # 4. boundary_bug
    solved = []
    for m, s in p0.items():
        r = s["by_case"].get("04_boundary_bug", {})
        stab = r.get("stability_rate")
        if stab and stab > 0:
            solved.append((m, stab))
    solved.sort(key=lambda x: x[1], reverse=True)
    solved_str = "・".join(f"{m}（{pct(st)}）" for m, st in solved)
    bullets.append(
        f"**boundary_bug はほぼ全モデルの壁** — "
        f"突破できたのは {len(solved)} モデルのみ: {solved_str}"
    )

    # 5. type_annotate の注意点
    type_fail = []
    type_ok = []
    for m, s in p1.items():
        r = s["by_case"].get("04_type_annotate", {})
        stab = r.get("stability_rate", 0) or 0
        t2 = r.get("t2", False)
        if not t2:
            type_fail.append(m)
        elif stab >= 0.5:
            type_ok.append(m)
    if type_fail:
        bullets.append(
            f"**type_annotate 失敗モデルに注意** — "
            f"{', '.join(type_fail)} は stab=0%。"
            f"代替: {', '.join(type_ok[:3])}"
        )

    # 6. P0高精度・P1不安定
    split = []
    for m in p0:
        s0 = p0[m]["stab"]
        s1v = p1.get(m, {}).get("stab", 0)
        t2_p0 = p0[m]["t2"]
        t2_p1 = p1.get(m, {}).get("t2", 0)
        if t2_p0 >= 6 and s0 >= 0.8 and t2_p1 >= 6 and s1v < 0.6:
            split.append((m, s0, s1v))
    for m, s0, s1v in split:
        bullets.append(
            f"**{m} は P0 高精度・P1 不安定** — "
            f"P0 stab={pct(s0)}（上位）/ P1 stab={pct(s1v)}。安定採用は要注意"
        )

    return [f"- {b}" for b in bullets]


def main():
    p0 = load_phase(P0_DIR)
    p1 = load_phase(P1_DIR)

    insights = generate_insights(p0, p1)
    if not insights:
        print("ERROR: insights を生成できませんでした", file=sys.stderr)
        sys.exit(1)

    content = README.read_text()
    new_section = "<!-- insights-start -->\n" + "\n".join(insights) + "\n<!-- insights-end -->"
    new_content = re.sub(
        r"<!-- insights-start -->.*?<!-- insights-end -->",
        new_section,
        content,
        flags=re.DOTALL,
    )

    if new_content == content:
        print("WARNING: マーカーが見つかりませんでした。README.md を更新しませんでした。", file=sys.stderr)
        sys.exit(1)

    README.write_text(new_content)
    print(f"✅ {len(insights)} 件の insights を README.md に書き込みました")
    for b in insights:
        print(f"  {b}")


if __name__ == "__main__":
    main()
