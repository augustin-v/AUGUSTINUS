# AUGUSTINUS (MVP → v1) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Raspberry Pi 5 (ARM64) “keyboard-only productivity OS feel” that boots to a 2x2 TUI (Motivation / Shell / AI Agents / Stats) with vim navigation, first-boot language selection, and durable persistence.

**Architecture:** A single fullscreen TUI binary (`augustinus`) runs on `tty1` under systemd. Internally it is an event-driven core (input, timers, PTY, metrics, AI I/O) feeding a pure rendering layer (ratatui). State is persisted via a small SQLite DB + TOML config, with a clear module boundary so panes can evolve into plugins later.

**Tech Stack:** Rust 2024, `ratatui` + `crossterm`, `tokio`, `portable-pty` + `vt100` (or `ratatui-term`), `serde` + `toml`, `sqlx` (sqlite) or `rusqlite`, `reqwest` (rustls), `tracing`.

---

## MVP Scope Definition (what “done” means)

**MVP must include:**
- Boot-to-app on `tty1` via systemd (no desktop, no mouse).
- Fullscreen animated ASCII “AUGUSTINUS” splash (2–3s) then app.
- First boot language selection (English / Français / 日本語), arrow navigation, white highlight, Enter confirm; persisted to config.
- 2x2 pane layout with titles: MOTIVATION / GENERAL / AI AGENTS / STATS.
- Keyboard controls: `h/j/k/l` focus, `Tab` rotate focus, `Enter` fullscreen focused pane, `Esc` exit fullscreen, `:` command mode (minimal: `:q`, `:help`, `:lang <en|fr|ja>`).
- GENERAL pane: embedded PTY shell (bash) with scrollback and basic ANSI/vt100 rendering.
- MOTIVATION pane: rotating message list (hardcoded for MVP), idle detection based on no input for N seconds, streak day counter (daily), focus session timer (start/stop via `:focus start|stop`).
- STATS pane: display streak days, focus time today, LOC changed today computed from `git diff --numstat` in configured repo path (or “N/A” if not a git repo yet).
- AI AGENTS pane: prompt input box, submit via `Ctrl-Enter`, append response (stubbed provider for MVP, real provider behind feature flag).

**Explicitly out of MVP (v1+):**
- Rich “tone modes”, subtle animations beyond splash + small indicators.
- Advanced resizing `H/J/K/L` beyond fixed grid (keep hooks, implement later).
- Multi-agent orchestration; only one “provider” interface.
- Calories/LOCK-IN editing UI (allow `:calories add <n>` command only).

---

## High-Level Phased Roadmap

### Phase 0 — Platform & Packaging
- Define repo/workspace layout, CI cross-build, systemd units, configuration paths.

### Phase 1 — Core TUI Shell (no PTY yet)
- Ratatui app loop, pane manager, input mapping, fullscreen, command mode, language selection flow.

### Phase 2 — Embedded PTY Shell (GENERAL)
- PTY spawn, terminal emulation render, scrollback, focus/resize integration.

### Phase 3 — Persistence & Metrics (MOTIVATION + STATS)
- Config + SQLite, streak/focus events, git diff parsing, idle detection.

### Phase 4 — AI Pane (stub → real)
- Provider trait, async request flow, secrets handling, UX polish.

### Phase 5 — Boot Experience Polish
- Splash animation tuning, first-boot UX, “sovereign OS” feel, crash recovery.

---

## Architecture Breakdown (concrete)

### Process model
- `augustinus` (single binary) owns:
  - **Input**: keyboard events from `crossterm`.
  - **Render**: ratatui draw at 30–60 FPS or “dirty” redraw.
  - **Timers**: tick (animation), idle timer.
  - **PTY**: background read/write for GENERAL.
  - **Storage**: config + db.
  - **AI**: background HTTP calls.

### Core modules (workspace layout)
- `crates/augustinus-app` (library): state machine + actions + reducers (testable, UI-agnostic).
- `crates/augustinus-tui` (library): ratatui widgets/panes, layout, theming.
- `crates/augustinus-pty` (library): PTY session + vt100 parser + scrollback.
- `crates/augustinus-store` (library): config (TOML) + db (SQLite) + repositories (metrics queries).
- `crates/augustinus-ai` (library): provider trait + HTTP implementations.
- `crates/augustinus-i18n` (library): strings per language + lookup helpers.
- `bin/augustinus` (binary): wiring, runtime, systemd/tty integration.

### Data model (durable)
- `config.toml` (human-editable): language, shell path, git repo path, theme.
- SQLite (append-only events + daily aggregates):
  - `events(id, ts, kind, payload_json)`
  - `daily(day, focus_seconds, streak_count, loc_added, loc_removed, calories, lock_in)`

### UI model
- App state is a single struct:
  - `focused_pane`, `fullscreen: Option<PaneId>`
  - per-pane state (motivation cycle, pty viewport, ai transcript, stats snapshot)
  - `command_mode: Option<CommandBuffer>`

---

## Recommended Rust Crates / Tools (by subsystem)

**TUI + input**
- `ratatui` (layout + widgets)
- `crossterm` (terminal backend + input)
- `unicode-width` (proper width)

**Async + logging**
- `tokio` (tasks/channels)
- `tracing`, `tracing-subscriber` (journald-friendly logs)
- `anyhow`, `thiserror` (errors)

**Config + persistence**
- `serde`, `toml`
- `directories` (XDG paths)
- SQLite: choose one:
  - **Option A (recommended)**: `sqlx` with `sqlite` + `runtime-tokio-rustls`
  - **Option B**: `rusqlite` + `spawn_blocking`

**PTY + terminal rendering**
- `portable-pty` (cross-platform PTY)
- `vt100` (escape parsing + screen model)
- Optional: `ratatui-term` (if it fits; otherwise keep custom widget)

**AI integration**
- `reqwest` + rustls (HTTP)
- `serde_json`
- Optional: `secrecy` (API keys)

**Cross build**
- `cross` (Docker-based), or `cargo-zigbuild` (no Docker)

---

## Boot Auto-Launch Strategy (systemd on minimal Linux)

**Target**: Raspberry Pi OS Lite / Debian minimal, boot to `tty1`, start `augustinus` as a systemd service that owns `/dev/tty1`.

Files to create:
- `packaging/systemd/augustinus.service`
- `packaging/systemd/augustinus.target` (optional)
- `packaging/systemd/README.md`
- `scripts/install-systemd.sh`

Service design:
- `Type=simple`
- `ExecStart=/usr/local/bin/augustinus`
- `TTYPath=/dev/tty1`, `StandardInput=tty`, `StandardOutput=tty`, `StandardError=journal`
- `Restart=always`, `RestartSec=1`
- Disable getty on tty1 by masking `getty@tty1.service` (documented in README/script)

First boot behavior:
- `augustinus` checks `config.toml`; if missing, run language selector, then write config.

---

## Risk Analysis (specific to this build)

1) **PTY/terminal emulation fidelity**: `vt100` may not render all modern TUIs perfectly. Mitigation: start with `bash + coreutils`, document limitations, consider swapping to `wezterm-term` later.
2) **Input handling conflicts**: shell apps expect raw mode; our TUI also uses raw mode. Mitigation: when GENERAL is focused, default to terminal input (locked) so typing works immediately; press `Esc` to return to app-controls for pane navigation; leaving GENERAL always resets to app-controls; avoid global shortcuts that interfere with terminal apps.
3) **Performance on ARM64**: full redraw at high FPS can stutter. Mitigation: dirty-render and cap FPS; use incremental updates for PTY.
4) **Data corruption on power loss**: SD cards are fragile. Mitigation: SQLite WAL mode, fsync tuning, buffered writes, clear “unsafe shutdown” detection.
5) **Secrets**: AI API keys must not be in logs. Mitigation: `secrecy` + redact logs; read key from `AUGUSTINUS_API_KEY` env var or a root-readable file.

---

## Phased Implementation Tasks (bite-sized, concrete)

### Task 1: Initialize Rust workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/augustinus-app/Cargo.toml`
- Create: `crates/augustinus-app/src/lib.rs`
- Create: `crates/augustinus-tui/Cargo.toml`
- Create: `crates/augustinus-tui/src/lib.rs`
- Create: `crates/augustinus-store/Cargo.toml`
- Create: `crates/augustinus-store/src/lib.rs`
- Create: `crates/augustinus-i18n/Cargo.toml`
- Create: `crates/augustinus-i18n/src/lib.rs`
- Create: `crates/augustinus-pty/Cargo.toml`
- Create: `crates/augustinus-pty/src/lib.rs`
- Create: `crates/augustinus-ai/Cargo.toml`
- Create: `crates/augustinus-ai/src/lib.rs`
- Create: `bin/augustinus/Cargo.toml`
- Create: `bin/augustinus/src/main.rs`

**Step 1: Write the failing build check**

Run: `cargo build -q`
Expected: FAIL (workspace missing)

**Step 2: Create minimal workspace**

Add root `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/*", "bin/*"]
```

Each crate: minimal lib/binary with `pub fn smoke() {}`.

**Step 3: Run build**

Run: `cargo build -q`
Expected: PASS

**Step 4: Commit**

```bash
git add Cargo.toml crates bin
git commit -m "chore: initialize rust workspace"
```

---

### Task 2: Add shared types: actions, panes, and app state

**Files:**
- Modify: `crates/augustinus-app/src/lib.rs`
- Create: `crates/augustinus-app/src/state.rs`
- Create: `crates/augustinus-app/src/action.rs`
- Create: `crates/augustinus-app/src/panes.rs`
- Test: `crates/augustinus-app/tests/reducer_focus.rs`

**Step 1: Write failing reducer test**

```rust
use augustinus_app::{Action, AppState, PaneId};

#[test]
fn hjkl_moves_focus_in_grid() {
    let mut s = AppState::new_for_test();
    assert_eq!(s.focused, PaneId::Motivation);
    s.apply(Action::FocusRight);
    assert_eq!(s.focused, PaneId::General);
    s.apply(Action::FocusDown);
    assert_eq!(s.focused, PaneId::Stats);
}
```

**Step 2: Run test**

Run: `cargo test -p augustinus-app -q`
Expected: FAIL (types missing)

**Step 3: Implement minimal state machine**

Implement:
- `PaneId::{Motivation, General, Agents, Stats}`
- `Action::{FocusLeft,FocusRight,FocusUp,FocusDown,RotateFocus,EnterFullscreen,ExitFullscreen,EnterCommandMode,ExitCommandMode}`
- `AppState { focused, fullscreen: Option<PaneId>, command: Option<String> }`
- `apply(Action)` with grid navigation:
  - TL(Motivation) ↔ TR(General)
  - BL(Agents) ↔ BR(Stats)

**Step 4: Run test**

Run: `cargo test -p augustinus-app -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/augustinus-app
git commit -m "feat(app): add focus/fullscreen reducers"
```

---

### Task 3: Implement TUI shell with arctic theme and 2x2 layout

**Files:**
- Modify: `crates/augustinus-tui/src/lib.rs`
- Create: `crates/augustinus-tui/src/theme.rs`
- Create: `crates/augustinus-tui/src/layout.rs`
- Create: `crates/augustinus-tui/src/panes/mod.rs`
- Create: `crates/augustinus-tui/src/panes/motivation.rs`
- Create: `crates/augustinus-tui/src/panes/general.rs`
- Create: `crates/augustinus-tui/src/panes/agents.rs`
- Create: `crates/augustinus-tui/src/panes/stats.rs`
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Write a manual smoke-run checklist (MVP)**

Create `docs/plans/notes-smoke.md` (temporary) with:
- App starts fullscreen, shows 4 titled panes
- Focus highlight changes on `h/j/k/l` and `Tab`

**Step 2: Implement ratatui frame**

In `bin/augustinus/src/main.rs`:
- enable raw mode + alternate screen
- init `Terminal<CrosstermBackend<Stdout>>`
- event loop: poll input + tick at ~30fps
- call `augustinus_tui::render(frame, &app_state)`

In theme:
- define colors: deep blue background, ice white foreground, pale cyan accents.
- focused pane border/title in ice white; unfocused in cyan.

**Step 3: Run locally**

Run: `cargo run -p augustinus -q`
Expected: fullscreen TUI with 4 panes and titles.

**Step 4: Commit**

```bash
git add bin/augustinus crates/augustinus-tui
git commit -m "feat(tui): render 2x2 layout with arctic theme"
```

---

### Task 4: Map keyboard controls to actions (vim + fullscreen + command mode)

**Files:**
- Modify: `bin/augustinus/src/main.rs`
- Modify: `crates/augustinus-app/src/action.rs`
- Modify: `crates/augustinus-app/src/state.rs`
- Test: `crates/augustinus-app/tests/reducer_fullscreen.rs`

**Step 1: Write failing fullscreen test**

```rust
use augustinus_app::{Action, AppState, PaneId};

#[test]
fn enter_and_exit_fullscreen() {
    let mut s = AppState::new_for_test();
    s.apply(Action::EnterFullscreen);
    assert_eq!(s.fullscreen, Some(PaneId::Motivation));
    s.apply(Action::ExitFullscreen);
    assert_eq!(s.fullscreen, None);
}
```

**Step 2: Run tests**

Run: `cargo test -p augustinus-app -q`
Expected: FAIL

**Step 3: Implement**
- `Enter` → `EnterFullscreen`
- `Esc` → `ExitFullscreen`
- `:` → `EnterCommandMode` (start empty buffer)
- While in command mode:
  - printable chars append
  - `Backspace` deletes
  - `Enter` submits command string (store last command; execution handled later)
  - `Esc` exits command mode without submit

**Step 4: Run**

Run: `cargo test -p augustinus-app -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/augustinus-app bin/augustinus
git commit -m "feat: add keymap for focus/fullscreen/command mode"
```

---

### Task 5: Add splash animation (ASCII “AUGUSTINUS”)

**Files:**
- Modify: `bin/augustinus/src/main.rs`
- Create: `crates/augustinus-tui/src/splash.rs`

**Step 1: Implement splash renderer**
- Render for a fixed duration (e.g., 2.5 seconds) before main UI.
- Use a pre-baked ASCII art constant for predictable layout.
- Animate with:
  - subtle horizontal shimmer (swap cyan/white per frame), or
  - fade-in lines over time.

**Step 2: Manual test**

Run: `cargo run -p augustinus -q`
Expected: animated “AUGUSTINUS” then main 2x2 UI.

**Step 3: Commit**

```bash
git add crates/augustinus-tui bin/augustinus
git commit -m "feat(boot): add animated AUGUSTINUS splash"
```

---

### Task 6: Implement first-boot language selection + persistence

**Files:**
- Modify: `crates/augustinus-store/src/lib.rs`
- Create: `crates/augustinus-store/src/config.rs`
- Modify: `crates/augustinus-i18n/src/lib.rs`
- Create: `crates/augustinus-i18n/src/strings.rs`
- Modify: `bin/augustinus/src/main.rs`
- Create: `crates/augustinus-tui/src/first_boot.rs`
- Test: `crates/augustinus-store/tests/config_roundtrip.rs`

**Step 1: Write failing config roundtrip test**

```rust
use augustinus_store::config::{AppConfig, Language};

#[test]
fn config_roundtrips_toml() {
    let c = AppConfig { language: Language::Ja, shell: "/bin/bash".into(), git_repo: None };
    let toml = c.to_toml_string();
    let parsed = AppConfig::from_toml_str(&toml).unwrap();
    assert_eq!(parsed.language, Language::Ja);
}
```

**Step 2: Run tests**

Run: `cargo test -p augustinus-store -q`
Expected: FAIL

**Step 3: Implement config**
- Config location via XDG: `$XDG_CONFIG_HOME/augustinus/config.toml` else `~/.config/augustinus/config.toml`.
- `Language` enum: `En`, `Fr`, `Ja`.
- `AppConfig { language, shell, git_repo }`.
- Provide `load_or_none()` and `save()`.

**Step 4: Implement first-boot selector UI**
- If config missing:
  - show list `[English, Français, 日本語]`
  - `Up/Down` arrows move selection
  - selected item styled in **white**
  - `Enter` persists config and continues

**Step 5: Run manually twice**

Run: `cargo run -p augustinus -q`
Expected first run: selector appears; second run: selector skipped.

**Step 6: Commit**

```bash
git add crates/augustinus-store crates/augustinus-i18n crates/augustinus-tui bin/augustinus
git commit -m "feat(i18n): first-boot language selection persisted to config"
```

---

### Task 7: Add MOTIVATION pane MVP (rotation + idle + streak counters in-memory)

**Files:**
- Modify: `crates/augustinus-app/src/state.rs`
- Modify: `crates/augustinus-tui/src/panes/motivation.rs`
- Create: `crates/augustinus-app/src/motivation.rs`
- Test: `crates/augustinus-app/tests/idle_detection.rs`

**Step 1: Write failing idle test**

```rust
use std::time::Duration;
use augustinus_app::motivation::IdleTracker;

#[test]
fn becomes_idle_after_threshold() {
    let mut t = IdleTracker::new(Duration::from_secs(5));
    t.on_activity();
    t.advance(Duration::from_secs(4));
    assert!(!t.is_idle());
    t.advance(Duration::from_secs(2));
    assert!(t.is_idle());
}
```

**Step 2: Run**

Run: `cargo test -p augustinus-app -q`
Expected: FAIL

**Step 3: Implement IdleTracker**
- `on_activity()` resets timer
- `advance(dt)` for test; in runtime use `Instant` deltas

**Step 4: Implement message rotation**
- hardcode ~20 messages (motivational + FOMO) for MVP
- rotate every N seconds (e.g., 15s)
- show idle badge if idle

**Step 5: Run**

Run: `cargo test -p augustinus-app -q`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/augustinus-app crates/augustinus-tui
git commit -m "feat(motivation): add rotating messages and idle detection"
```

---

### Task 8: Add SQLite storage and daily aggregates

**Files:**
- Modify: `crates/augustinus-store/Cargo.toml`
- Modify: `crates/augustinus-store/src/lib.rs`
- Create: `crates/augustinus-store/src/db.rs`
- Create: `crates/augustinus-store/migrations/001_init.sql`
- Test: `crates/augustinus-store/tests/db_smoke.rs`

**Step 1: Write failing db smoke test**

```rust
use augustinus_store::db::Store;

#[test]
fn creates_schema_and_inserts_event() {
    let store = Store::open_in_memory().unwrap();
    store.insert_event("focus_start", "{}").unwrap();
    let n = store.count_events().unwrap();
    assert_eq!(n, 1);
}
```

**Step 2: Run**

Run: `cargo test -p augustinus-store -q`
Expected: FAIL

**Step 3: Implement Store**
- `open(path)` + `open_in_memory()` for tests
- apply migration `001_init.sql`
- `events` + `daily` tables
- WAL mode enabled for on-disk DB

**Step 4: Run**

Run: `cargo test -p augustinus-store -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/augustinus-store
git commit -m "feat(store): add sqlite schema for events and daily stats"
```

---

### Task 9: Implement focus sessions + streak persistence

**Files:**
- Modify: `crates/augustinus-app/src/state.rs`
- Modify: `bin/augustinus/src/main.rs`
- Modify: `crates/augustinus-store/src/db.rs`
- Create: `crates/augustinus-app/src/focus.rs`

**Step 1: Implement commands**
- `:focus start` → insert `focus_start` event, mark session active
- `:focus stop` → insert `focus_stop` event, compute elapsed, accumulate `daily.focus_seconds`

**Step 2: Implement streak**
- On each app start, compute streak based on `daily` rows with nonzero focus.
- Display in MOTIVATION + STATS.

**Step 3: Manual test**
- Start focus, wait 10s, stop, verify STATS shows ~10s for today.

**Step 4: Commit**

```bash
git add crates/augustinus-app crates/augustinus-store bin/augustinus
git commit -m "feat(focus): persist focus sessions and streak"
```

---

### Task 10: Add GENERAL pane PTY (portable-pty + vt100)

**Files:**
- Modify: `crates/augustinus-pty/Cargo.toml`
- Modify: `crates/augustinus-pty/src/lib.rs`
- Create: `crates/augustinus-pty/src/session.rs`
- Modify: `crates/augustinus-tui/src/panes/general.rs`
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Implement PTY session**
- Spawn configured shell (default `/bin/bash`) with `portable-pty`.
- Background task reads PTY output, feeds bytes into `vt100::Parser`.
- Provide API:
  - `send_key(KeyEvent)`
  - `resize(cols, rows)`
  - `snapshot()` → screen cells for rendering

**Step 2: Render screen in GENERAL pane**
- Convert vt100 screen to ratatui `Text` with proper colors if available.
- Maintain scrollback ring buffer (MVP: last N lines).

**Step 3: Manual test**
- Run `cargo run -p augustinus -q`
- Focus GENERAL, run `ls`, `python3 --version`, `cargo --version`.

**Step 4: Commit**

```bash
git add crates/augustinus-pty crates/augustinus-tui bin/augustinus
git commit -m "feat(pty): embed shell in GENERAL pane"
```

---

### Task 11: Add STATS LOC parsing via git diff

**Files:**
- Modify: `crates/augustinus-app/src/state.rs`
- Create: `crates/augustinus-app/src/stats.rs`
- Modify: `crates/augustinus-tui/src/panes/stats.rs`
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Implement repo path config**
- `AppConfig.git_repo` optional.
- If set, run `git diff --numstat` in that directory on a timer (e.g., every 30s).

**Step 2: Parse `--numstat`**
- Sum added/removed; ignore binary `-` entries.
- Display `+added / -removed` in STATS.

**Step 3: Commit**

```bash
git add crates/augustinus-app crates/augustinus-tui bin/augustinus
git commit -m "feat(stats): show loc changes from git diff --numstat"
```

---

### Task 12: Add AI AGENTS pane (stub provider)

**Files:**
- Modify: `crates/augustinus-ai/src/lib.rs`
- Create: `crates/augustinus-ai/src/provider.rs`
- Create: `crates/augustinus-ai/src/stub.rs`
- Modify: `crates/augustinus-tui/src/panes/agents.rs`
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Define provider trait**
- `send(prompt) -> Future<Output=Result<String>>`

**Step 2: Implement stub**
- returns canned response with timestamp and echo of prompt.

**Step 3: UI**
- input box at bottom
- transcript above (scroll later; MVP: truncate)
- submit with `Ctrl-Enter`

**Step 4: Commit**

```bash
git add crates/augustinus-ai crates/augustinus-tui bin/augustinus
git commit -m "feat(ai): add agents pane with stub provider"
```

---

### Task 13: Add real AI provider behind feature flag

**Files:**
- Modify: `crates/augustinus-ai/Cargo.toml`
- Modify: `crates/augustinus-ai/src/lib.rs`
- Create: `crates/augustinus-ai/src/openai_compatible.rs`
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Feature-flag**
- `--features ai-openai` enables reqwest dependency.

**Step 2: Env-based secret**
- Read `AUGUSTINUS_API_KEY`.
- Fail gracefully with on-screen error (no panic).

**Step 3: Commit**

```bash
git add crates/augustinus-ai bin/augustinus
git commit -m "feat(ai): add openai-compatible provider behind feature flag"
```

---

### Task 14: Systemd unit + installer script

**Files:**
- Create: `packaging/systemd/augustinus.service`
- Create: `packaging/systemd/README.md`
- Create: `scripts/install-systemd.sh`

**Step 1: Write unit file**
- Own `tty1`, restart on crash.
- Document masking `getty@tty1.service`.

**Step 2: Write install script**
- copy binary to `/usr/local/bin/augustinus`
- install unit to `/etc/systemd/system/`
- `systemctl enable --now augustinus.service`

**Step 3: Commit**

```bash
git add packaging scripts
git commit -m "feat(packaging): add systemd autostart for tty1"
```

---

### Task 15: Hardening & polish (MVP exit criteria)

**Files:**
- Modify: `bin/augustinus/src/main.rs`
- Modify: `crates/augustinus-tui/src/theme.rs`

**Steps:**
- Ensure raw mode cleanup on panic (panic hook + drop guard).
- Add `:q` to exit cleanly (for dev).
- Cap redraw rate and avoid CPU spin.
- Ensure focused border is always ice white, and highlight matches requirements.

**Commit**

```bash
git add bin/augustinus crates/augustinus-tui
git commit -m "chore: polish runtime cleanup and performance caps"
```

---

## v1+ Extensions (after MVP)
- Implement `H/J/K/L` resizing with persistent layout ratios in config.
- Add calories + LOCK IN editing UI and historical charts.
- Add plugin registry for panes and “agents” backends.
- Replace vt100 stack if needed for richer terminal apps.
