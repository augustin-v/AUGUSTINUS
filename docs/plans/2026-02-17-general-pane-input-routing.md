# General Pane Input Routing + `^` Leader Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Stop `h/j/k/l/Tab/:/Enter/Esc` from being sent into the GENERAL PTY by default, and replace the current `Ctrl-Space` “leader” with a Mac-friendly `^` keybind, while still allowing full terminal interaction when desired.

**Architecture:** Introduce an explicit input routing mode in `AppState` (app-controls vs terminal-pass-through). Default is **app-controls**, so pane navigation always works even when GENERAL is focused. Press `^` (Shift+6) while GENERAL is focused to toggle terminal pass-through on/off. TUI displays the current mode and the new hint.

**Tech Stack:** Existing `crossterm` key events + current `PtySession` routing; minimal state additions in `augustinus-app`.

---

## Current Implementation (verified)
- `bin/augustinus/src/main.rs` currently sends **all keys** to PTY whenever `state.focused == PaneId::General` unless a `leader_armed` flag was set by `Ctrl-Space`. This makes `j/k` type into the shell instead of moving focus.
- `crates/augustinus-tui/src/panes/general.rs` prints a hint referencing `Ctrl-Space`.

---

## Target UX (acceptance criteria)

1) Focus GENERAL, press `j` / `k`:
- Focus moves between panes; nothing is typed into the PTY.

2) Focus GENERAL, press `^`:
- Enters “terminal pass-through” mode (indicator visible in GENERAL header/hint).

3) In terminal pass-through mode:
- Regular typing works, including `h/j/k/l`, `Esc`, `Tab`, `:` (they go to the PTY).
- Press `^` again to exit pass-through mode back to app-controls.

4) Switching focus away from GENERAL:
- Always exits terminal pass-through mode automatically (prevents “stuck in terminal mode”).

5) Remove `Ctrl-Space` from UI hints and docs.

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
fn toggling_general_mode_works() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::General;
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalPassthrough);
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn leaving_general_resets_to_app_mode() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::General;
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalPassthrough);
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
- `pub enum GeneralInputMode { AppControls, TerminalPassthrough }`

Add to `AppState` in `crates/augustinus-app/src/state.rs`:
- `pub general_input_mode: GeneralInputMode`
- default to `AppControls` in `new_for_test()`

Add to `Action` in `crates/augustinus-app/src/action.rs`:
- `ToggleGeneralInputMode`

Reducer rules in `AppState::apply`:
- `ToggleGeneralInputMode` only toggles when `focused == PaneId::General` (no-op otherwise).
- Any action that changes focus away from GENERAL (FocusLeft/Right/Up/Down/RotateFocus) must:
  - compute the new focused pane
  - if new focused is not GENERAL: force `general_input_mode = AppControls`

**Step 4: Run tests**

Run: `cargo test -p augustinus-app -q`
Expected: PASS

**Step 5: Commit**
```bash
git add crates/augustinus-app
git commit -m "feat(app): add general input routing mode"
```

---

## Task 2: Replace `Ctrl-Space` leader with `^` routing in runtime key handling

**Files:**
- Modify: `bin/augustinus/src/main.rs`

**Step 1: Remove old leader behavior**
- Delete `leader_armed` state and the `Ctrl-Space` check.

**Step 2: Implement new routing precedence**

Update `handle_key(...)` logic (high-level order):
1. If `state.command.is_some()` → command editing (unchanged).
2. If `key.code == KeyCode::Char('^')` and `key.modifiers.is_empty()`:
   - `state.apply(Action::ToggleGeneralInputMode)` and return.
3. If `state.focused == PaneId::General` and `state.general_input_mode == TerminalPassthrough`:
   - send key to `pty.send_key(key)` and return.
4. Otherwise:
   - handle app keybinds (`h/j/k/l`, `Tab`, `Enter`, `Esc`, `:`) as today.

**Step 3: Control-C quit behavior**
- Only treat `Ctrl-C` as “quit app” when **not** in terminal pass-through.
- In terminal pass-through, forward `Ctrl-C` to the PTY (so shell programs still work).

**Step 4: Manual verification**

Run: `cargo run -p augustinus -q`
Checklist:
- Focus GENERAL, `j/k` moves focus.
- Press `^`, type `ls`, it appears and runs.
- Press `^` again, `j/k` moves focus again.

**Step 5: Commit**
```bash
git add bin/augustinus
git commit -m "feat(input): route general keys via ^ toggle instead of ctrl-space"
```

---

## Task 3: Update GENERAL pane hint + show current mode

**Files:**
- Modify: `crates/augustinus-tui/src/panes/general.rs`

**Step 1: Update hint text**
- Replace the first line with something like:
  - `^ toggles TERMINAL INPUT (pass-through)`
  - `App controls: h/j/k/l Tab : Enter Esc`

**Step 2: Show mode indicator**
- If `state.general_input_mode == TerminalPassthrough`, show a visible “PASS-THROUGH” indicator near the top (e.g., a colored label line).

**Step 3: Manual verification**

Run: `cargo run -p augustinus -q`
Expected:
- The hint references `^`, not `Ctrl-Space`.
- Indicator flips when pressing `^` in GENERAL.

**Step 4: Commit**
```bash
git add crates/augustinus-tui
git commit -m "chore(tui): update general pane hint and mode indicator"
```

---

## Task 4: Update existing plan doc to avoid stale `Ctrl-Space` references

**Files:**
- Modify: `docs/plans/2026-02-17-augustinus-mvp.md`

**Step 1: Replace text**
- Update the “Input handling conflicts” mitigation to reference `^` and/or the new routing mode.

**Step 2: Commit**
```bash
git add docs/plans/2026-02-17-augustinus-mvp.md
git commit -m "docs: update keybind references to ^"
```

---

## Notes / Follow-ups (optional, not required for this fix)
- Consider a 3-state model later: `AppControls`, `TerminalPassthrough`, `TerminalPassthroughWithLeader` (tmux-like prefix) if you want to keep a way to trigger app commands without fully leaving terminal mode.

