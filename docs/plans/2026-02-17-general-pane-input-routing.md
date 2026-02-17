# General Pane Input Routing + Terminal Lock (Esc unlock) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** When focus moves to GENERAL, typing works immediately (keys go to the PTY). While in terminal-locked mode, `h/j/k/l` must not move focus; pressing `Esc` returns to app-controls so pane navigation works again. Leaving GENERAL always resets to app-controls.

**Architecture:** Keep a single input routing source of truth in `AppState` via `GeneralInputMode::{AppControls, TerminalLocked}`. Reducer rules set `TerminalLocked` whenever focus becomes GENERAL, and force `AppControls` whenever focus becomes non-GENERAL. Runtime key handling in `bin/augustinus` forwards keys to the PTY while terminal-locked (except `Esc`, which exits terminal lock).

**Tech Stack:** Existing `crossterm` key events + current `PtySession` routing; minimal state additions in `augustinus-app`.

---

## Current Implementation (verified)
- Input routing is modeled in `GeneralInputMode` and enforced in `bin/augustinus` key handling.
- GENERAL UI hints must reflect terminal lock + `Esc` unlock.

---

## Target UX (acceptance criteria)

1) Move focus to GENERAL:
- Regular typing works immediately (keys go to the PTY).
- While terminal-locked, pressing `h/j/k/l` does **not** move focus; they go to the PTY.

2) Press `Esc` while focused on GENERAL:
- Exits terminal-locked mode back to app-controls (pane navigation works again).

3) Switching focus away from GENERAL:
- Always forces app-controls (prevents “stuck in terminal mode”).

---

## Task 1: Add explicit input routing state to `AppState`

**Files:**
- Modify: `crates/augustinus-app/src/state.rs`
- Modify: `crates/augustinus-app/src/action.rs`
- Create: `crates/augustinus-app/src/input.rs`
- Modify: `crates/augustinus-app/src/lib.rs`
- Test: `crates/augustinus-app/tests/general_input_routing.rs`

**Step 1: Write failing tests**

Create `crates/augustinus-app/tests/general_input_routing.rs`:
```rust
use augustinus_app::{Action, AppState, GeneralInputMode, PaneId};

#[test]
fn general_starts_in_app_mode() {
    let s = AppState::new_for_test();
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn entering_general_sets_terminal_locked() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusRight); // Motivation -> General
    assert_eq!(s.focused, PaneId::General);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalLocked);
}

#[test]
fn leaving_general_resets_to_app_mode() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusRight); // Motivation -> General
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalLocked);
    s.apply(Action::FocusLeft); // General -> Motivation
    assert_eq!(s.focused, PaneId::Motivation);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}
```

**Step 2: Run tests**

Run: `cargo test -p augustinus-app -q`
Expected: FAIL (missing `GeneralInputMode` + action + behavior)

**Step 3: Implement minimal state + reducer behavior**

Implement in `crates/augustinus-app/src/input.rs`:
- `pub enum GeneralInputMode { AppControls, TerminalLocked }`

Add to `AppState` in `crates/augustinus-app/src/state.rs`:
- `pub general_input_mode: GeneralInputMode`
- default to `AppControls` in `new_for_test()`

Add to `Action` in `crates/augustinus-app/src/action.rs`:
- `EnterGeneralTerminalMode`
- `ExitGeneralTerminalMode`

Reducer rules in `AppState::apply`:
- Any action that changes focus (FocusLeft/Right/Up/Down/RotateFocus) must:
  - compute the new focused pane
  - if new focused is GENERAL: force `general_input_mode = TerminalLocked`
  - otherwise: force `general_input_mode = AppControls`
- `ExitGeneralTerminalMode` only affects GENERAL and sets `AppControls`.
- `EnterGeneralTerminalMode` only affects GENERAL and sets `TerminalLocked`.

**Step 4: Run tests**

Run: `cargo test -p augustinus-app -q`
Expected: PASS

**Step 5: Commit**
```bash
git add crates/augustinus-app
git commit -m "feat(app): add general input routing mode"
```

---

## Task 2: Terminal lock routing in runtime key handling

**Files:**
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Implement routing precedence**

Update `handle_key(...)` logic (high-level order):
1. If `state.command.is_some()` → command editing (unchanged).
2. If `state.focused == PaneId::General` and `state.general_input_mode == TerminalLocked`:
   - if key is `Esc`: `state.apply(Action::ExitGeneralTerminalMode)` and return.
   - otherwise: send key to `pty.send_key(key)` and return.
3. Otherwise:
   - handle app keybinds (`h/j/k/l`, `Tab`, `Enter`, `Esc`, `:`) as today.

**Step 3: Control-C quit behavior**
- Only treat `Ctrl-C` as “quit app” when **not** in terminal-locked mode.
- In terminal-locked mode, forward `Ctrl-C` to the PTY (so shell programs still work).

**Step 2: Manual verification**

Run: `cargo run -p augustinus -q`
Checklist:
- Focus GENERAL: typing works immediately; `h/j/k/l` do not move focus.
- Press `Esc`: unlocks; now `h/j/k/l` moves focus again.

**Step 5: Commit**
```bash
git add bin/augustinus
git commit -m "feat(input): lock general terminal input until Esc"
```

---

## Task 3: Update GENERAL pane hint + show current mode

**Files:**
- Modify: `crates/augustinus-tui/src/panes/general.rs`

**Step 1: Update hint text**
- When terminal-locked:
  - `TERMINAL MODE (locked) — Esc to return to app controls`
- Otherwise:
  - `Press Enter to fullscreen; Focus with h/j/k/l; ":" commands`

**Step 2: Show mode indicator**
- If `state.general_input_mode == TerminalLocked`, show a visible “TERMINAL MODE (locked)” indicator near the top (e.g., a colored label line).

**Step 3: Manual verification**

Run: `cargo run -p augustinus -q`
Expected:
- The hint references `Esc` for unlocking terminal input.
- Indicator flips when pressing `Esc` in GENERAL.

**Step 4: Commit**
```bash
git add crates/augustinus-tui
git commit -m "chore(tui): update general pane hint and mode indicator"
```

---

## Task 4: Update existing plan doc to avoid stale keybind references

**Files:**
- Modify: `docs/plans/2026-02-17-augustinus-mvp.md`

**Step 1: Replace text**
- Update the “Input handling conflicts” mitigation to reference terminal lock + `Esc` unlock.

**Step 2: Commit**
```bash
git add docs/plans/2026-02-17-augustinus-mvp.md
git commit -m "docs: update keybind docs for terminal lock mode"
```

---

## Notes / Follow-ups (optional, not required for this fix)
- Consider a 3-state model later: `AppControls`, `TerminalLocked`, `TerminalLockedWithLeader` (tmux-like prefix) if you want a way to trigger app commands without fully leaving terminal mode.
