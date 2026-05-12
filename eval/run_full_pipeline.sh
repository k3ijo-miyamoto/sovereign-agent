#!/bin/bash
set -e
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export SOVEREIGN_BIN="$REPO_ROOT/rust/target/debug/sovereign"
cd "$REPO_ROOT"

MODELS=(
  gemma3:27b gemma3:12b gemma3:4b
  mistral-nemo:12b
  qwen3:8b-nothink qwen3:14b qwen3:8b
  codestral:22b
  llama3.1:8b granite3.3:8b
  phi4:14b
  deepseek-coder-v2:16b
  qwen2.5-coder:14b qwen2.5:7b
  devstral:24b
)

echo "===== Phase 0 START: $(date) ====="
for model in "${MODELS[@]}"; do
  echo "--- Phase 0: $model ---"
  python3 eval/phase0/run_eval.py --model "$model" --runs 3 --no-docker-warn
done
echo "===== Phase 0 DONE: $(date) ====="

echo "===== Phase 1 START: $(date) ====="
for model in "${MODELS[@]}"; do
  echo "--- Phase 1: $model ---"
  python3 eval/phase1/run_eval.py --model "$model" --runs 3 --no-docker-warn
done
echo "===== Phase 1 DONE: $(date) ====="

echo "===== Summarize ====="
python3 eval/phase0/summarize.py -o eval/phase0/summary.md --update-readme
python3 eval/phase1/summarize.py -o eval/phase1/summary.md --update-readme

echo "===== Update insights ====="
echo "NOTE: README.md の insights セクションは Claude Code による更新が必要です。"
echo "      以下のプロンプトを Claude Code に貼り付けてください:"
echo "----"
cat eval/prompts/update_insights.txt
echo "----"

echo "===== ALL DONE: $(date) ====="
