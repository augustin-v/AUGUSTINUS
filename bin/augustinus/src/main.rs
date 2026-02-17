use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use augustinus_app::{Action, AppState};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut state = AppState::new_for_test();
    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            augustinus_tui::render(frame, &state);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if should_quit(key) {
                    break;
                }
                if let Some(action) = key_to_action(key) {
                    state.apply(action);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn should_quit(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn key_to_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('h') => Some(Action::FocusLeft),
        KeyCode::Char('j') => Some(Action::FocusDown),
        KeyCode::Char('k') => Some(Action::FocusUp),
        KeyCode::Char('l') => Some(Action::FocusRight),
        KeyCode::Tab => Some(Action::RotateFocus),
        _ => None,
    }
}

