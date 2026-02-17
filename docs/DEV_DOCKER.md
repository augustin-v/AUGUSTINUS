# Docker (ARM64 / Pi-like) Dev Workflow

This repo includes a Docker setup that runs `augustinus` in a `linux/arm64` container to catch ARM-specific build/runtime issues early (useful for Raspberry Pi-style deployments).

## Prereqs

- Docker Desktop (or Docker Engine) with `docker compose`

## Build + run

```bash
docker compose build
docker compose run --rm augustinus
```

## Notes

- QEMU emulation can be slow; expect builds to take longer than native.
- The AI AGENTS pane will try to run `codex`. If `codex` isn’t present, the container entrypoint will attempt `npm i -g @openai/codex`.
- If you want to control the command used for the Agents PTY, set `agents_cmd` in the app config (`~/.config/augustinus/config.toml`).

