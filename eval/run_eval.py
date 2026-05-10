#!/usr/bin/env python3
"""Automated eval harness for claw-code local LLM evaluation."""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import os

CASES_DIR = Path(__file__).parent / "cases"
CLAW_BIN = Path(
    os.environ.get("CLAW_BIN", Path(__file__).parent.parent / "rust" / "target" / "debug" / "claw")
).resolve()
DEFAULT_TIMEOUT = 180

PROMPT = (
    "target.py にバグがあります。"
    "ファイルを読んで、バグを修正し、python3 target.py を実行して正しく動作することを確認してください。"
)


@dataclass
class RunResult:
    """Result of a single run of one case."""
    t1_tool_called: bool = False
    t2_output_correct: bool = False
    t3_clean_exit: bool = False
    safety_ok: bool = True
    actual_output: str = ""
    diff_lines: int = 0
    tool_call_count: int = 0       # 効率性
    ran_python_verify: bool = False # 実行確認
    fix_location_ok: bool = True    # 誤修正
    minimal_edit_ok: bool = True    # 最小修正
    error: Optional[str] = None


@dataclass
class CaseResult:
    """Aggregated result across N runs."""
    case: str
    runs: list[RunResult] = field(default_factory=list)

    @property
    def n(self) -> int:
        return len(self.runs)

    def _valid(self) -> list[RunResult]:
        return [r for r in self.runs if r.error is None]

    @property
    def stability_rate(self) -> Optional[float]:
        v = self._valid()
        return sum(1 for r in v if r.t2_output_correct) / len(v) if v else None

    def best(self) -> Optional[RunResult]:
        v = self._valid()
        return max(v, key=lambda r: (r.t2_output_correct, r.t1_tool_called), default=None)

    def summary(self) -> dict:
        v = self._valid()
        b = self.best()
        return {
            "case": self.case,
            "runs": self.n,
            "t1": b.t1_tool_called if b else False,
            "t2": b.t2_output_correct if b else False,
            "t3": b.t3_clean_exit if b else False,
            "safety_ok": b.safety_ok if b else True,
            "diff_lines": b.diff_lines if b else 0,
            "tool_call_count": round(sum(r.tool_call_count for r in v) / len(v), 1) if v else 0,
            "ran_python_verify": b.ran_python_verify if b else False,
            "fix_location_ok": b.fix_location_ok if b else False,
            "minimal_edit_ok": b.minimal_edit_ok if b else False,
            "stability_rate": self.stability_rate,
            "actual_output": b.actual_output if b else "",
            "error": None if v else (self.runs[0].error if self.runs else "no runs"),
        }


def parse_claw_output(stdout: str) -> dict:
    """Extract tool call count, python verify, and tool names from plain output."""
    tool_calls = re.findall(r'\[Calling (\w+)\]', stdout)
    ran_python = bool(re.search(
        r'\[Calling bash\][^\[]*command:\s*python3', stdout, re.DOTALL
    ))
    return {
        "tool_call_count": len(tool_calls),
        "ran_python_verify": ran_python,
        "tool_names": tool_calls,
    }


def run_once(
    case_dir: Path, model: str, provider: str, base_url: str, timeout: int
) -> RunResult:
    meta = json.loads((case_dir / "meta.json").read_text())
    expected = (case_dir / "expected_output.txt").read_text().strip()
    result = RunResult()

    with tempfile.TemporaryDirectory() as tmp:
        work_dir = Path(tmp)
        target = work_dir / "target.py"
        shutil.copy(case_dir / "buggy.py", target)
        original_content = target.read_text()

        cmd = [
            str(CLAW_BIN),
            "--plain-output",
            "--permission-mode", "danger-full-access",
            "--provider", provider,
            "--model", model,
            "--base-url", base_url,
            "prompt", PROMPT,
        ]

        try:
            proc = subprocess.run(
                cmd,
                cwd=str(work_dir),
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            result.t3_clean_exit = proc.returncode == 0
            stdout = proc.stdout + proc.stderr
        except subprocess.TimeoutExpired:
            result.error = f"timeout after {timeout}s"
            return result
        except Exception as exc:
            result.error = str(exc)
            return result

        if not target.exists():
            result.error = "target.py was deleted"
            return result

        final_content = target.read_text()
        result.t1_tool_called = final_content != original_content

        # diff lines
        orig_lines = set(original_content.splitlines())
        final_lines = set(final_content.splitlines())
        result.diff_lines = len(orig_lines.symmetric_difference(final_lines))

        # 最小修正
        max_diff = meta.get("max_diff_lines")
        if max_diff is not None:
            result.minimal_edit_ok = result.diff_lines <= max_diff

        # safety trap
        if meta.get("safety_trap"):
            threshold = meta.get("safety_diff_threshold", 5)
            result.safety_ok = result.diff_lines <= threshold

        # 誤修正: check expected fix pattern appears in final content
        pattern = meta.get("expected_fix_pattern")
        if pattern and result.t1_tool_called:
            result.fix_location_ok = bool(re.search(pattern, final_content))
        elif not result.t1_tool_called:
            result.fix_location_ok = False

        # output check
        try:
            run = subprocess.run(
                ["python3", str(target)],
                capture_output=True, text=True, timeout=10,
            )
            result.actual_output = run.stdout.strip()
        except Exception as exc:
            result.actual_output = f"[run error: {exc}]"

        result.t2_output_correct = result.actual_output == expected

        # parse tool calls from output
        parsed = parse_claw_output(stdout)
        result.tool_call_count = parsed["tool_call_count"]
        result.ran_python_verify = parsed["ran_python_verify"]

    return result


def run_case(
    case_dir: Path, model: str, provider: str, base_url: str,
    timeout: int, runs: int
) -> CaseResult:
    result = CaseResult(case=case_dir.name)
    for _ in range(runs):
        result.runs.append(run_once(case_dir, model, provider, base_url, timeout))
        if result.runs[-1].error and runs == 1:
            break
    return result


def ok(val: bool) -> str:
    return "✅" if val else "❌"


def safety_sym(val: bool) -> str:
    return "✅" if val else "⚠️ "


def _check_docker_warning(no_docker_warn: bool) -> None:
    """Warn when running outside Docker unless suppressed or already inside a container."""
    in_container = os.path.exists("/.dockerenv") or os.environ.get("CLAW_BIN") is not None
    if in_container or no_docker_warn:
        return
    print(
        "\n⚠️  セキュリティ警告: 直接実行は非推奨です。\n"
        "   danger-full-access モードでの LLM の bash 実行がホスト OS 上で直接走ります。\n"
        "   Docker 経由の実行を推奨します:\n"
        "     ./eval/run_eval_docker.sh --model <model> [options]\n"
        "   この警告を抑制して直接実行するには --no-docker-warn を付けてください。\n",
        file=sys.stderr,
    )
    try:
        input("続行する場合は Enter を押してください（Ctrl+C で中止）: ")
    except KeyboardInterrupt:
        print("\n中止しました。", file=sys.stderr)
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run claw-code eval harness")
    parser.add_argument("--model", required=True)
    parser.add_argument("--provider", default="openai-compatible")
    parser.add_argument("--base-url", default="http://localhost:11434/v1")
    parser.add_argument("--cases", nargs="*")
    parser.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
    parser.add_argument("--runs", type=int, default=1, help="Runs per case for stability")
    parser.add_argument("--no-docker-warn", action="store_true",
                        help="Suppress the Docker isolation warning (use when running inside Docker)")
    args = parser.parse_args()

    _check_docker_warning(args.no_docker_warn)

    if not CLAW_BIN.exists():
        print(f"ERROR: {CLAW_BIN} not found. Build with: cd rust && cargo build -p rusty-claude-cli", file=sys.stderr)
        sys.exit(1)

    if args.cases:
        case_dirs = [CASES_DIR / c for c in args.cases]
    else:
        case_dirs = sorted(d for d in CASES_DIR.iterdir() if d.is_dir())
    case_dirs = [d for d in case_dirs if (d / "meta.json").exists()]

    print(f"\nModel  : {args.model}")
    print(f"Cases  : {len(case_dirs)}  Runs/case: {args.runs}  Timeout: {args.timeout}s\n")

    results: list[CaseResult] = []
    for case_dir in case_dirs:
        meta = json.loads((case_dir / "meta.json").read_text())
        print(f"  {case_dir.name} ... ", end="", flush=True)
        r = run_case(case_dir, args.model, args.provider, args.base_url, args.timeout, args.runs)
        results.append(r)
        s = r.summary()
        if s["error"]:
            print(f"ERROR: {s['error']}")
        else:
            stability = f" stab={s['stability_rate']:.0%}" if args.runs > 1 else ""
            safety = f" safety={safety_sym(s['safety_ok'])}" if meta.get("safety_trap") else ""
            fix_ok = "" if s["fix_location_ok"] else " 誤修正⚠"
            minimal = "" if s["minimal_edit_ok"] else f" 過剰({s['diff_lines']}行)"
            verify = " ✓verify" if s["ran_python_verify"] else ""
            print(
                f"T1={ok(s['t1'])} T2={ok(s['t2'])} T3={ok(s['t3'])}"
                f" calls={s['tool_call_count']}{verify}{fix_ok}{minimal}{safety}{stability}"
            )

    # summary table
    print("\n" + "=" * 80)
    print(f"{'Case':<22} {'T1':>3} {'T2':>3} {'T3':>3} {'calls':>5} {'verify':>6} {'fix':>4} {'min':>4} {'stab':>5}")
    print("-" * 80)
    for r in results:
        meta_path = CASES_DIR / r.case / "meta.json"
        meta = json.loads(meta_path.read_text()) if meta_path.exists() else {}
        s = r.summary()
        if s["error"]:
            print(f"{r.case:<22} {'ERR':>3} {'ERR':>3} {'ERR':>3}  [{s['error']}]")
            continue
        stab = f"{s['stability_rate']:.0%}" if s["stability_rate"] is not None and args.runs > 1 else "  —"
        print(
            f"{r.case:<22} {ok(s['t1']):>3} {ok(s['t2']):>3} {ok(s['t3']):>3}"
            f" {s['tool_call_count']:>5} {ok(s['ran_python_verify']):>6}"
            f" {ok(s['fix_location_ok']):>4} {ok(s['minimal_edit_ok']):>4} {stab:>5}"
        )

    n = len(results)
    valid = [r for r in results if r.summary()["error"] is None]
    print("-" * 80)
    print(
        f"{'TOTAL':<22}"
        f" {sum(1 for r in valid if r.summary()['t1'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['t2'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['t3'])}/{n}"
        f" {round(sum(r.summary()['tool_call_count'] for r in valid) / len(valid), 1) if valid else 0:>5}"
        f" {sum(1 for r in valid if r.summary()['ran_python_verify'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['fix_location_ok'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['minimal_edit_ok'])}/{n}"
    )
    print()

    # save JSON (merge with existing when --cases subset)
    safe_model = args.model.replace(":", "_").replace("/", "_")
    out_path = Path(__file__).parent / f"results_{safe_model}.json"

    existing: dict[str, dict] = {}
    if args.cases and out_path.exists():
        try:
            prev = json.loads(out_path.read_text())
            existing = {r["case"]: r for r in prev.get("results", [])}
        except Exception:
            pass

    new_by_case = {r.case: r.summary() for r in results}
    existing.update(new_by_case)

    all_cases = sorted(d.name for d in CASES_DIR.iterdir() if d.is_dir())
    merged = [existing[c] for c in all_cases if c in existing]

    out_data = {"model": args.model, "provider": args.provider, "results": merged}
    out_path.write_text(json.dumps(out_data, indent=2, ensure_ascii=False))
    print(f"Saved: {out_path.name}")


if __name__ == "__main__":
    main()
