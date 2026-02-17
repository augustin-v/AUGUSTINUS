#!/usr/bin/env bash
set -euo pipefail

if ! command -v codex >/dev/null 2>&1; then
  if command -v npm >/dev/null 2>&1; then
    echo "codex not found; installing @openai/codex via npm..." >&2
    npm i -g @openai/codex >/dev/null
  fi
fi

exec augustinus

