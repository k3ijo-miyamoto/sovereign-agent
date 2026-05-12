#!/usr/bin/env python3
"""Summarize Phase 1 eval results from results/*.json into a Markdown table."""

import argparse
import json
import re
from datetime import date
from pathlib import Path

EVAL_DIR = Path(__file__).parent
CASES_DIR = EVAL_DIR / "cases"
README_PATH = Path(__file__).parent.parent.parent / "README.md"

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


def load_results(path: Path) -> dict:
    data = json.loads(path.read_text())
    return {"model": data["model"], "by_case": {r["case"]: r for r in data["results"]}}


def load_criteria(case: str) -> dict:
    path = CASES_DIR / case / "expected_criteria.json"
    if path.exists():
        return json.loads(path.read_text())
    return {}


def model_stats(entry: dict, case_dirs: list[str]) -> dict:
    valid = [entry["by_case"][c] for c in case_dirs
             if c in entry["by_case"] and not entry["by_case"][c].get("error")]
    calls = [r["tool_call_count"] for r in valid if r.get("tool_call_count") is not None]
    stab  = [r["stability_rate"]  for r in valid if r.get("stability_rate")  is not None]
    return {
        "model":     entry["model"],
        "n":         len(valid),
        "t2":        sum(1 for r in valid if r["t2"]),
        "t3":        sum(1 for r in valid if r["t3"]),
        "avg_calls": round(sum(calls) / len(calls), 1) if calls else None,
        "avg_stab":  round(sum(stab)  / len(stab),  2) if stab  else None,
    }


def note(s: dict) -> str:
    if s["n"] == 0:
        return "データなし"
    if s["t2"] == s["n"]:
        return "全冠"
    if s["t2"] == 0:
        return "実用不可"
    return ""


def generate_summary(all_results: list[dict], case_dirs: list[str]) -> list[str]:
    stats  = [model_stats(e, case_dirs) for e in all_results]
    ranked = sorted(stats, key=lambda s: (s["t2"], s["avg_stab"] or 0, s["t3"]), reverse=True)
    n      = len(case_dirs)
    lines: list[str] = []

    lines.append("## 集計表")
    lines.append("")
    lines.append("| モデル | T2 | T3 | avg calls | stab(avg) | 特記 |")
    lines.append("|---|:---:|:---:|:---:|:---:|---|")
    for s in ranked:
        calls = f"{s['avg_calls']}" if s["avg_calls"] is not None else "—"
        stab  = f"{s['avg_stab']:.0%}" if s["avg_stab"] is not None else "—"
        lines.append(f"| {s['model']} | **{s['t2']}/{n}** | {s['t3']}/{n} | {calls} | {stab} | {note(s)} |")
    lines.append("")
    return lines


def generate_case_table(all_results: list[dict], case_dirs: list[str]) -> list[str]:
    bases = [c.split("_", 1)[1] if "_" in c else c for c in case_dirs]
    counts: dict[str, int] = {}
    for b in bases:
        counts[b] = counts.get(b, 0) + 1
    short = [b if counts[b] == 1 else c for b, c in zip(bases, case_dirs)]
    lines: list[str] = []

    lines.append("## ケース別 T2 結果")
    lines.append("")
    lines.append("| モデル | " + " | ".join(short) + " |")
    lines.append("|---|" + ":---:|" * len(case_dirs))

    sorted_results = sorted(
        all_results,
        key=lambda e: model_stats(e, case_dirs)["t2"],
        reverse=True,
    )
    for entry in sorted_results:
        cells = []
        for c in case_dirs:
            r = entry["by_case"].get(c)
            if r is None or r.get("error"):
                cells.append("—")
            elif r["t2"]:
                stab = r.get("stability_rate")
                cells.append(f"✅{stab:.0%}" if stab is not None else "✅")
            else:
                cells.append("❌")
        lines.append(f"| {entry['model']} | " + " | ".join(cells) + " |")

    lines.append("")
    lines.append("> ✅ = T2通過（安定性）、❌ = T2失敗")
    lines.append("")
    return lines


def generate_checks_detail(all_results: list[dict], case_dirs: list[str], top_n: int = 4) -> list[str]:
    top_entries = sorted(
        all_results,
        key=lambda e: model_stats(e, case_dirs)["t2"],
        reverse=True,
    )[:top_n]
    top_models = [e["model"] for e in top_entries]
    lines: list[str] = []

    bases = [c.split("_", 1)[1] if "_" in c else c for c in case_dirs]
    base_counts: dict[str, int] = {}
    for b in bases:
        base_counts[b] = base_counts.get(b, 0) + 1
    case_shorts = {c: (b if base_counts[b] == 1 else c) for b, c in zip(bases, case_dirs)}

    lines.append(f"## ケース別 checks 詳細（上位{top_n}モデル）")
    lines.append("")

    for case in case_dirs:
        short = case_shorts[case]
        criteria = load_criteria(case)
        check_descriptions = {c["id"]: c.get("description", c["id"]) for c in criteria.get("checks", [])}
        t2_required = set(criteria.get("scoring", {}).get("t2", []))

        # Collect check keys in criteria order, then any extras from results
        check_keys: list[str] = [c["id"] for c in criteria.get("checks", [])]
        for e in top_entries:
            for k in e["by_case"].get(case, {}).get("checks", {}).keys():
                if k not in check_keys:
                    check_keys.append(k)

        if not check_keys:
            continue

        lines.append(f"### {short}")
        lines.append("")
        lines.append("| check | 説明 | T2必須 | " + " | ".join(top_models) + " |")
        lines.append("|---|---|:---:|" + ":---:|" * len(top_models))

        for ck in check_keys:
            desc    = check_descriptions.get(ck, "")
            req     = "✓" if ck in t2_required else ""
            cells   = []
            for e in top_entries:
                val = e["by_case"].get(case, {}).get("checks", {}).get(ck)
                cells.append("✅" if val else ("❌" if val is False else "—"))
            lines.append(f"| `{ck}` | {desc} | {req} | " + " | ".join(cells) + " |")

        lines.append("")

    return lines


def _readme_p1_table(all_results: list[dict], case_dirs: list[str]) -> str:
    stats = [model_stats(e, case_dirs) for e in all_results]
    n = len(case_dirs)
    top = sorted(
        [s for s in stats if s["n"] > 0 and s["t2"] > n // 2],
        key=lambda s: (s["t2"], s["avg_stab"] or 0),
        reverse=True,
    )[:8]

    docstring_count = sum(1 for c in case_dirs if "docstring" in c)
    lines = [
        f"> docstring 追加は{docstring_count}サブケース（単純・複雑・ヒント付き）に分けて評価。他タスクは各1ケース。",
        "",
        "| モデル | ケース通過 | サイズ |",
        "|---|:---:|---:|",
    ]

    for s in top:
        size = MODEL_SIZES.get(s["model"], "?")
        name = f"**{s['model']}**" if s["t2"] == n else s["model"]
        lines.append(f"| {name} | {s['t2']}/{n} | {size}GB |")

    return "\n".join(lines)


def _replace_readme_section(readme_path: Path, marker: str, new_content: str) -> bool:
    text = readme_path.read_text()
    pattern = re.compile(
        rf"(<!-- {re.escape(marker)}-start -->)\n.*?\n(<!-- {re.escape(marker)}-end -->)",
        re.DOTALL,
    )
    if not pattern.search(text):
        print(f"Warning: marker '{marker}' not found in {readme_path}")
        return False
    updated = pattern.sub(rf"\1\n{new_content}\n\2", text)
    readme_path.write_text(updated)
    return True


def update_readme_p1(readme_path: Path, all_results: list[dict], case_dirs: list[str]) -> None:
    ok = _replace_readme_section(readme_path, "eval-p1", _readme_p1_table(all_results, case_dirs))
    print(f"README Phase 1 updated: {ok}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize Phase 1 eval results")
    parser.add_argument("--output", "-o", help="Write Markdown to this file (default: stdout)")
    parser.add_argument(
        "--results-dir",
        default=str(EVAL_DIR / "results"),
        help="Directory containing *.json result files",
    )
    parser.add_argument("--top", type=int, default=4, help="Number of models in checks detail (default: 4)")
    parser.add_argument(
        "--update-readme",
        action="store_true",
        help="Update Phase 1 tables in README.md",
    )
    parser.add_argument(
        "--readme",
        default=str(README_PATH),
        help="Path to README.md (default: project root)",
    )
    args = parser.parse_args()

    results_dir = Path(args.results_dir)
    result_files = sorted(results_dir.glob("*.json"))
    if not result_files:
        print("No *.json files found.")
        return

    all_results = [load_results(f) for f in result_files]

    case_dirs = sorted(d.name for d in CASES_DIR.iterdir() if d.is_dir())
    all_cases_in_results = {c for e in all_results for c in e["by_case"]}
    case_dirs = [c for c in case_dirs if c in all_cases_in_results]
    sc_bases = [c.split("_", 1)[1] if "_" in c else c for c in case_dirs]
    sc_counts: dict[str, int] = {}
    for b in sc_bases:
        sc_counts[b] = sc_counts.get(b, 0) + 1
    short_cases = [b if sc_counts[b] == 1 else c for b, c in zip(sc_bases, case_dirs)]

    lines: list[str] = []
    lines.append("# Phase 1 評価サマリ（実務タスク・全モデル）")
    lines.append("")
    lines.append(f"> 更新日: {date.today().isoformat()}")
    lines.append(f"> ハーネス: `eval/phase1/run_eval.py`")
    lines.append(f"> ケース数: {len(case_dirs)}（{', '.join(short_cases)}）")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines += generate_summary(all_results, case_dirs)
    lines += generate_case_table(all_results, case_dirs)
    lines += generate_checks_detail(all_results, case_dirs, top_n=args.top)

    output = "\n".join(lines)
    if args.output:
        Path(args.output).write_text(output)
        print(f"Written to {args.output}")
    else:
        print(output)

    if args.update_readme:
        update_readme_p1(Path(args.readme), all_results, case_dirs)


if __name__ == "__main__":
    main()
