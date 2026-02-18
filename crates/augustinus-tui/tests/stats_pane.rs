use augustinus_app::{AppState, LocDelta};
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn stats_pane_renders_cards_and_status_table() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new_for_test();
    state.focus.set_streak_days(7);
    state.focus.set_focus_seconds_today(3_661);
    state.loc_delta = Some(LocDelta { added: 12, removed: 3 });

    terminal
        .draw(|f| augustinus_tui::render(f, &mut state))
        .unwrap();

    let buf = terminal.backend().buffer();
    let screen = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Outer pane title.
    assert!(screen.contains("STATS"));

    // New “full” layout sections (these do not exist in the current sparse implementation).
    assert!(screen.contains("STREAK"));
    assert!(screen.contains("FOCUS"));
    assert!(screen.contains("LOC"));
    assert!(screen.contains("STATUS"));
}

#[test]
fn stats_pane_small_terminal_still_renders_useful_text() {
    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new_for_test();
    state.focus.set_streak_days(1);
    state.focus.set_focus_seconds_today(60);

    terminal
        .draw(|f| augustinus_tui::render(f, &mut state))
        .unwrap();

    let buf = terminal.backend().buffer();
    let screen = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show *some* compact stats, not blank space.
    assert!(screen.contains("Streak") || screen.contains("STREAK"));
    assert!(screen.contains("Focus") || screen.contains("FOCUS"));
}
