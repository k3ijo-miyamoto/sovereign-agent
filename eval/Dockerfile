FROM python:3.12-slim

# Minimal eval environment — no extra packages needed (eval harness uses stdlib only)
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pytest==8.3.5

RUN useradd -m -u 1000 eval

WORKDIR /app

# claw binary is mounted at runtime (see run_eval_docker.sh)
# eval/ directory is mounted at runtime for cases and results output

USER eval
ENTRYPOINT ["python3"]
