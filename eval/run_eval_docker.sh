#!/usr/bin/env bash
# Run the eval harness inside a Docker container for OS-level isolation.
# The LLM operates with danger-full-access inside the container, so any
# unintended bash commands are confined to the container's filesystem.
#
# Usage:
#   ./eval/run_eval_docker.sh [--phase1] --model qwen3:8b [run_eval.py options...]
#
# Requirements:
#   - Docker running
#   - Ollama running on the host (localhost:11434)
#   - sovereign binary built: cargo build -p sovereign

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOVEREIGN_BIN="${REPO_ROOT}/rust/target/debug/sovereign"
IMAGE_NAME="sovereign-eval"

# --phase1 フラグを先に抜き出す
PHASE1=false
PASSTHROUGH=()
for arg in "$@"; do
  if [[ "${arg}" == "--phase1" ]]; then
    PHASE1=true
  else
    PASSTHROUGH+=("${arg}")
  fi
done

if [[ ! -f "${SOVEREIGN_BIN}" ]]; then
  echo "ERROR: sovereign binary not found at ${SOVEREIGN_BIN}"
  echo "Build with: cd rust && cargo build -p sovereign"
  exit 1
fi

# Build image if not present or if Dockerfile changed
if ! docker image inspect "${IMAGE_NAME}" &>/dev/null || \
   [[ "${REPO_ROOT}/eval/Dockerfile" -nt "${REPO_ROOT}/eval/.docker_build_stamp" ]]; then
  echo "Building Docker image ${IMAGE_NAME}..."
  docker build -t "${IMAGE_NAME}" "${REPO_ROOT}/eval"
  touch "${REPO_ROOT}/eval/.docker_build_stamp"
fi

if [[ "${PHASE1}" == true ]]; then
  EVAL_SCRIPT="run_eval_phase1.py"
else
  EVAL_SCRIPT="run_eval.py"
fi

echo "Running ${EVAL_SCRIPT} in Docker container (isolated)..."
docker run --rm \
  --network host \
  --read-only \
  --tmpfs /tmp \
  --tmpfs /home/eval \
  --security-opt no-new-privileges \
  --cap-drop ALL \
  -e SOVEREIGN_BIN=/usr/local/bin/sovereign \
  -v "${SOVEREIGN_BIN}:/usr/local/bin/sovereign:ro" \
  -v "${REPO_ROOT}/eval:/app/eval" \
  "${IMAGE_NAME}" python3 "/app/eval/${EVAL_SCRIPT}" --base-url http://localhost:11434 "${PASSTHROUGH[@]}"
