#!/usr/bin/env bash
# Run the eval harness inside a Docker container for OS-level isolation.
# The LLM operates with danger-full-access inside the container, so any
# unintended bash commands are confined to the container's filesystem.
#
# Usage:
#   ./eval/run_eval_docker.sh --model gemma3:27b [run_eval.py options...]
#
# Requirements:
#   - Docker running
#   - Ollama running on the host (localhost:11434)
#   - claw binary built: cargo build -p rusty-claude-cli

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLAW_BIN="${REPO_ROOT}/rust/target/debug/claw"
IMAGE_NAME="claw-eval"

if [[ ! -f "${CLAW_BIN}" ]]; then
  echo "ERROR: claw binary not found at ${CLAW_BIN}"
  echo "Build with: cargo build -p rusty-claude-cli"
  exit 1
fi

# Build image if not present or if Dockerfile changed
if ! docker image inspect "${IMAGE_NAME}" &>/dev/null || \
   [[ "${REPO_ROOT}/eval/Dockerfile" -nt <(docker image inspect "${IMAGE_NAME}" --format '{{.Metadata.LastTagTime}}' 2>/dev/null || echo "") ]]; then
  echo "Building Docker image ${IMAGE_NAME}..."
  docker build -t "${IMAGE_NAME}" "${REPO_ROOT}/eval"
fi

echo "Running eval in Docker container (isolated)..."
docker run --rm \
  --network host \
  --read-only \
  --tmpfs /tmp \
  --tmpfs /home/eval \
  --security-opt no-new-privileges \
  --cap-drop ALL \
  -e CLAW_BIN=/usr/local/bin/claw \
  -v "${CLAW_BIN}:/usr/local/bin/claw:ro" \
  -v "${REPO_ROOT}/eval:/app/eval" \
  "${IMAGE_NAME}" --base-url http://host.docker.internal:11434/v1 "$@"
