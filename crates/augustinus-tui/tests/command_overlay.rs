use augustinus_app::AppState;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn command_overlay_renders_when_active() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new_for_test();
    state.command = Some("q".to_string());

    terminal.draw(|f| augustinus_tui::render(f, &mut state)).unwrap();

    let buf = terminal.backend().buffer();
    let screen = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(screen.contains(":q"));
}
