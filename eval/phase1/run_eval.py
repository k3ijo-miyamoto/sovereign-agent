#!/usr/bin/env python3
"""Phase1 eval harness — criteria-based evaluation (docstring, test gen, etc.)"""

import argparse
import ast
import datetime
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
SOVEREIGN_BIN = Path(
    os.environ.get("SOVEREIGN_BIN", Path(__file__).parent.parent / "rust" / "target" / "debug" / "sovereign")
).resolve()
DEFAULT_TIMEOUT = 180

EDIT_TOOLS = {"edit", "edit_file", "write", "write_file"}
READ_TOOLS = {"read", "read_file", "glob", "grep"}


# ---------------------------------------------------------------------------
# Check implementations
# ---------------------------------------------------------------------------

def _check_compile(source: str) -> tuple[bool, str]:
    try:
        compile(source, "<string>", "exec")
        return True, "ok"
    except SyntaxError as e:
        return False, str(e)


def _body_nodes(func: ast.FunctionDef) -> list[ast.stmt]:
    """Return function body, skipping the leading docstring if present."""
    body = func.body
    if (body
            and isinstance(body[0], ast.Expr)
            and isinstance(body[0].value, ast.Constant)
            and isinstance(body[0].value.value, str)):
        return body[1:]
    return body


def _check_no_body_change(original: str, modified: str) -> tuple[bool, str]:
    """Ensure all function bodies are unchanged (ignoring docstrings)."""
    try:
        orig_tree = ast.parse(original)
        mod_tree = ast.parse(modified)
    except SyntaxError as e:
        return False, f"SyntaxError: {e}"

    orig_funcs = {n.name: n for n in ast.walk(orig_tree) if isinstance(n, ast.FunctionDef)}
    mod_funcs = {n.name: n for n in ast.walk(mod_tree) if isinstance(n, ast.FunctionDef)}

    for fname, orig_func in orig_funcs.items():
        if fname not in mod_funcs:
            return False, f"Function '{fname}' missing after edit"
        orig_dump = ast.dump(ast.Module(body=_body_nodes(orig_func), type_ignores=[]))
        mod_dump = ast.dump(ast.Module(body=_body_nodes(mod_funcs[fname]), type_ignores=[]))
        if orig_dump != mod_dump:
            return False, f"Function '{fname}' body changed"
    return True, "All function bodies unchanged"


def _get_all_docstrings(source: str) -> str:
    """Return all docstrings concatenated (for keyword search)."""
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return ""
    parts = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Module)):
            ds = ast.get_docstring(node)
            if ds:
                parts.append(ds)
    return " ".join(parts)


def _check_docstring_added(source: str) -> tuple[bool, str]:
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        return False, f"SyntaxError: {e}"
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            if ast.get_docstring(node):
                return True, f"Docstring found in '{node.name}'"
    return False, "No function docstring found"


def _check_string_match(source: str, keywords: list[str]) -> tuple[bool, str]:
    docstrings = _get_all_docstrings(source)
    text = docstrings.lower()
    found = [kw for kw in keywords if kw.lower() in text]
    if found:
        return True, f"Matched: {found}"
    return False, f"None of {keywords} found in docstrings"


def _check_source_match(source: str, keywords: list[str]) -> tuple[bool, str]:
    """Search keywords in the full source text (not just docstrings)."""
    text = source.lower()
    found = [kw for kw in keywords if kw.lower() in text]
    return (True, f"Matched: {found}") if found else (False, f"None of {keywords} found in source")


def _check_annotations_added(source: str) -> tuple[bool, str]:
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        return False, f"SyntaxError: {e}"
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            has_return = node.returns is not None
            has_args = any(arg.annotation for arg in node.args.args)
            if has_return or has_args:
                return True, f"Annotations found in '{node.name}'"
    return False, "No type annotations found in any function"


def _check_all_annotated(source: str) -> tuple[bool, str]:
    """Check that every function has both parameter and return type annotations."""
    try:
        tree = ast.parse(source)
    except SyntaxError as e:
        return False, f"SyntaxError: {e}"
    missing = []
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            if node.returns is None:
                missing.append(f"'{node.name}' missing return annotation")
            unannotated = [a.arg for a in node.args.args if a.annotation is None]
            if unannotated:
                missing.append(f"'{node.name}' missing param annotations: {unannotated}")
    if missing:
        return False, "; ".join(missing)
    return True, "All functions fully annotated"


def _check_file_exists(work_dir: Path, filename: str) -> tuple[bool, str]:
    path = work_dir / filename
    return (True, f"{filename} exists") if path.exists() else (False, f"{filename} not found")


def _check_pytest_pass(work_dir: Path, filename: str) -> tuple[bool, str]:
    path = work_dir / filename
    if not path.exists():
        return False, f"{filename} not found"
    try:
        r = subprocess.run(
            ["python3", "-m", "pytest", filename, "-v", "--tb=short"],
            cwd=str(work_dir), capture_output=True, text=True, timeout=60,
        )
        if r.returncode == 0:
            return True, "All tests passed"
        return False, (r.stdout + r.stderr)[-600:]
    except Exception as exc:
        return False, str(exc)


def _check_string_match_file(work_dir: Path, filename: str, keywords: list[str]) -> tuple[bool, str]:
    path = work_dir / filename
    if not path.exists():
        return False, f"{filename} not found"
    text = path.read_text().lower()
    found = [kw for kw in keywords if kw.lower() in text]
    return (True, f"Matched: {found}") if found else (False, f"None of {keywords} found")


def _check_first_line_max_length(work_dir: Path, filename: str, max_length: int) -> tuple[bool, str]:
    path = work_dir / filename
    if not path.exists():
        return False, f"{filename} not found"
    first_line = path.read_text().split("\n")[0]
    if len(first_line) <= max_length:
        return True, f"Subject {len(first_line)} chars (≤ {max_length})"
    return False, f"Subject too long: {len(first_line)} chars (> {max_length})"


def run_check(check_def: dict, original: str, modified: str, work_dir: Path = None) -> tuple[bool, str]:
    method = check_def["method"]
    if method == "compile":
        return _check_compile(modified)
    if method == "ast":
        return _check_docstring_added(modified)
    if method == "diff":
        return _check_no_body_change(original, modified)
    if method == "string_match":
        return _check_string_match(modified, check_def.get("keywords", []))
    if method == "source_match":
        return _check_source_match(modified, check_def.get("keywords", []))
    if method == "ast_annotations":
        return _check_annotations_added(modified)
    if method == "ast_all_annotated":
        return _check_all_annotated(modified)
    if method == "file_exists":
        return _check_file_exists(work_dir, check_def["filename"]) if work_dir else (False, "work_dir not provided")
    if method == "pytest_pass":
        return _check_pytest_pass(work_dir, check_def.get("filename", "test_target.py")) if work_dir else (False, "work_dir not provided")
    if method == "string_match_file":
        return _check_string_match_file(work_dir, check_def["filename"], check_def.get("keywords", [])) if work_dir else (False, "work_dir not provided")
    if method == "first_line_max_length":
        return _check_first_line_max_length(work_dir, check_def["filename"], check_def.get("max_length", 72)) if work_dir else (False, "work_dir not provided")
    return False, f"Unknown method: {method}"


# ---------------------------------------------------------------------------
# Action sequence parsing
# ---------------------------------------------------------------------------

def parse_action_sequence(stdout: str) -> dict:
    tool_calls = re.findall(r"\[Calling (\w+)\]", stdout)
    did_read = any(t in READ_TOOLS for t in tool_calls)
    did_edit = any(t in EDIT_TOOLS for t in tool_calls)
    edit_calls = [t for t in tool_calls if t in EDIT_TOOLS]
    did_re_edit = len(edit_calls) > 1
    did_verify = bool(re.search(
        r"\[Calling bash\][^\[]*python3", stdout, re.DOTALL
    ))
    return {
        "tool_call_count": len(tool_calls),
        "tool_names": tool_calls,
        "did_read": did_read,
        "did_edit": did_edit,
        "did_verify": did_verify,
        "did_re_edit": did_re_edit,
    }


# ---------------------------------------------------------------------------
# Run
# ---------------------------------------------------------------------------

@dataclass
class RunResult:
    checks: dict = field(default_factory=dict)
    check_details: dict = field(default_factory=dict)
    t1: bool = False
    t2: bool = False
    t3: bool = False
    tool_call_count: int = 0
    tool_names: list = field(default_factory=list)
    did_read: bool = False
    did_edit: bool = False
    did_verify: bool = False
    did_re_edit: bool = False
    error: Optional[str] = None


@dataclass
class CaseResult:
    case: str
    runs: list[RunResult] = field(default_factory=list)

    @property
    def n(self):
        return len(self.runs)

    def _valid(self):
        return [r for r in self.runs if r.error is None]

    def best(self) -> Optional[RunResult]:
        v = self._valid()
        return max(v, key=lambda r: (r.t2, r.t1), default=None)

    @property
    def stability_rate(self) -> Optional[float]:
        v = self._valid()
        return sum(1 for r in v if r.t2) / len(v) if v else None

    def summary(self) -> dict:
        v = self._valid()
        b = self.best()
        return {
            "case": self.case,
            "runs": self.n,
            "t1": b.t1 if b else False,
            "t2": b.t2 if b else False,
            "t3": b.t3 if b else False,
            "checks": b.checks if b else {},
            "tool_call_count": round(sum(r.tool_call_count for r in v) / len(v), 1) if v else 0,
            "did_read": b.did_read if b else False,
            "did_edit": b.did_edit if b else False,
            "did_verify": b.did_verify if b else False,
            "did_re_edit": b.did_re_edit if b else False,
            "stability_rate": self.stability_rate,
            "error": None if v else (self.runs[0].error if self.runs else "no runs"),
        }


def run_once(
    case_dir: Path, model: str, provider: str, base_url: str, timeout: int
) -> RunResult:
    criteria = json.loads((case_dir / "expected_criteria.json").read_text())
    prompt = (case_dir / "prompt.txt").read_text().strip()
    scoring = criteria.get("scoring", {})
    check_defs = {c["id"]: c for c in criteria.get("checks", [])}
    result = RunResult()

    with tempfile.TemporaryDirectory() as tmp:
        work_dir = Path(tmp)

        # Copy all case files to work_dir (target.py may not exist for non-Python tasks)
        skip = {"expected_criteria.json", "prompt.txt"}
        for src in case_dir.iterdir():
            if src.name not in skip:
                shutil.copy(src, work_dir / src.name)

        target = work_dir / "target.py"
        original = target.read_text() if target.exists() else ""

        cmd = [
            str(SOVEREIGN_BIN),
            "--plain-output",
            "--permission-mode", "danger-full-access",
            "--provider", provider,
            "--model", model,
            "--base-url", base_url,
            "prompt", prompt,
        ]

        try:
            proc = subprocess.run(
                cmd, cwd=str(work_dir),
                capture_output=True, text=True, timeout=timeout,
            )
            stdout = proc.stdout + proc.stderr
        except subprocess.TimeoutExpired:
            result.error = f"timeout after {timeout}s"
            return result
        except Exception as exc:
            result.error = str(exc)
            return result

        if original and not target.exists():
            result.error = "target.py was deleted"
            return result

        modified = target.read_text() if target.exists() else ""

        # Run all checks
        for check_id, check_def in check_defs.items():
            ok, detail = run_check(check_def, original, modified, work_dir=work_dir)
            result.checks[check_id] = ok
            result.check_details[check_id] = detail

        # Score T1/T2/T3
        result.t1 = all(result.checks.get(c, False) for c in scoring.get("t1", []))
        result.t2 = all(result.checks.get(c, False) for c in scoring.get("t2", []))
        result.t3 = all(result.checks.get(c, False) for c in scoring.get("t3", []))

        # Action sequence
        seq = parse_action_sequence(stdout)
        result.tool_call_count = seq["tool_call_count"]
        result.tool_names = seq["tool_names"]
        result.did_read = seq["did_read"]
        result.did_edit = seq["did_edit"]
        result.did_verify = seq["did_verify"]
        result.did_re_edit = seq["did_re_edit"]

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


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

def ok(val: bool) -> str:
    return "✅" if val else "❌"


def seq_str(r: dict) -> str:
    parts = []
    if r["did_read"]:   parts.append("R")
    if r["did_edit"]:   parts.append("E")
    if r["did_verify"]: parts.append("V")
    if r["did_re_edit"]: parts.append("Re")
    return "→".join(parts) if parts else "—"


def _check_docker_warning(no_docker_warn: bool) -> None:
    in_container = os.path.exists("/.dockerenv") or os.environ.get("SOVEREIGN_BIN") is not None
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
    parser = argparse.ArgumentParser(description="Run claw-code phase1 eval harness")
    parser.add_argument("--model", required=True)
    parser.add_argument("--provider", default="ollama")
    parser.add_argument("--base-url", default="http://localhost:11434")
    parser.add_argument("--cases", nargs="*")
    parser.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
    parser.add_argument("--runs", type=int, default=1)
    parser.add_argument("--no-docker-warn", action="store_true")
    args = parser.parse_args()

    _check_docker_warning(args.no_docker_warn)

    if not SOVEREIGN_BIN.exists():
        print(f"ERROR: {SOVEREIGN_BIN} not found. Build with: cd rust && cargo build -p sovereign", file=sys.stderr)
        sys.exit(1)

    if args.cases:
        case_dirs = [CASES_DIR / c for c in args.cases]
    else:
        case_dirs = sorted(d for d in CASES_DIR.iterdir() if d.is_dir())
    case_dirs = [d for d in case_dirs if (d / "expected_criteria.json").exists()]

    print(f"\nModel  : {args.model}")
    print(f"Cases  : {len(case_dirs)}  Runs/case: {args.runs}  Timeout: {args.timeout}s\n")

    results: list[CaseResult] = []
    for case_dir in case_dirs:
        criteria = json.loads((case_dir / "expected_criteria.json").read_text())
        print(f"  {case_dir.name} ({criteria.get('task', '?')}) ... ", end="", flush=True)
        r = run_case(case_dir, args.model, args.provider, args.base_url, args.timeout, args.runs)
        results.append(r)
        s = r.summary()
        if s["error"]:
            print(f"ERROR: {s['error']}")
        else:
            stab = f" stab={s['stability_rate']:.0%}" if args.runs > 1 else ""
            print(
                f"T1={ok(s['t1'])} T2={ok(s['t2'])} T3={ok(s['t3'])}"
                f" calls={s['tool_call_count']} seq={seq_str(s)}{stab}"
            )
            # show per-check detail
            for check_id, passed in s["checks"].items():
                print(f"    {ok(passed)} {check_id}")

    # Summary table
    n = len(results)
    valid = [r for r in results if r.summary()["error"] is None]
    print("\n" + "=" * 80)
    print(f"{'Case':<30} {'T1':>3} {'T2':>3} {'T3':>3} {'calls':>5} {'seq':<12} {'stab':>5}")
    print("-" * 80)
    for r in results:
        s = r.summary()
        if s["error"]:
            print(f"{r.case:<30} {'ERR':>3} {'ERR':>3} {'ERR':>3}  [{s['error']}]")
            continue
        stab = f"{s['stability_rate']:.0%}" if s["stability_rate"] is not None and args.runs > 1 else "  —"
        print(
            f"{r.case:<30} {ok(s['t1']):>3} {ok(s['t2']):>3} {ok(s['t3']):>3}"
            f" {s['tool_call_count']:>5} {seq_str(s):<12} {stab:>5}"
        )
    print("-" * 80)
    print(
        f"{'TOTAL':<30}"
        f" {sum(1 for r in valid if r.summary()['t1'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['t2'])}/{n}"
        f" {sum(1 for r in valid if r.summary()['t3'])}/{n}"
        f" {round(sum(r.summary()['tool_call_count'] for r in valid) / len(valid), 1) if valid else 0:>5}"
    )
    print()

    # Save JSON
    safe_model = args.model.replace(":", "_").replace("/", "_")
    out_path = Path(__file__).parent / "results" / f"{safe_model}.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)

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

    # sovereign git hash
    try:
        git_hash = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=Path(__file__).parent.parent,
            text=True,
        ).strip()
    except Exception:
        git_hash = "unknown"

    # ollama version
    try:
        ollama_ver = subprocess.check_output(["ollama", "--version"], text=True).strip()
    except Exception:
        ollama_ver = "unknown"

    out_data = {
        "model": args.model,
        "provider": args.provider,
        "evaluated_at": datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat(),
        "runs_per_case": args.runs,
        "timeout_sec": args.timeout,
        "sovereign_binary": str(SOVEREIGN_BIN),
        "sovereign_git": git_hash,
        "ollama_version": ollama_ver,
        "results": merged,
    }
    out_path.write_text(json.dumps(out_data, indent=2, ensure_ascii=False))
    print(f"Saved: {out_path.name}")


if __name__ == "__main__":
    main()
