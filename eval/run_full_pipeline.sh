#!/bin/bash
set -e
export SOVEREIGN_BIN=/path/to/sovereign-agent/rust/target/debug/sovereign
cd /path/to/sovereign-agent

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
PROMPT=$(cat eval/prompts/update_insights.txt)
"$SOVEREIGN_BIN" --model qwen3:14b --plain-output prompt "$PROMPT"

echo "===== ALL DONE: $(date) ====="
