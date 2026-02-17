use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use augustinus_app::{Action, AppState};
use augustinus_store::config::{AppConfig, Language};
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

    let result = (|| {
        run_splash(&mut terminal, Duration::from_millis(2500))?;
        let config = AppConfig::load_or_none().map_err(anyhow_to_io)?;
        if config.is_none() {
            let language = run_first_boot(&mut terminal)?;
            let config = AppConfig {
                language,
                shell: "/bin/bash".to_string(),
                git_repo: None,
            };
            let _ = config.save().map_err(anyhow_to_io)?;
        }
        run_app(&mut terminal)
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn anyhow_to_io(error: anyhow::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, error)
}

fn run_splash(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    duration: Duration,
) -> io::Result<()> {
    let start = Instant::now();
    let tick_rate = Duration::from_millis(33);

    while start.elapsed() < duration {
        let elapsed = start.elapsed();
        terminal.draw(|frame| {
            augustinus_tui::render_splash(frame, elapsed);
        })?;

        let timeout = tick_rate;
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if should_quit(key) {
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

fn run_first_boot(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<Language> {
    let mut selected_index: usize = 0;
    let tick_rate = Duration::from_millis(33);

    loop {
        terminal.draw(|frame| {
            augustinus_tui::render_first_boot(frame, selected_index);
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if should_quit(key) {
                    return Ok(Language::En);
                }
                match key.code {
                    KeyCode::Up => selected_index = selected_index.saturating_sub(1),
                    KeyCode::Down => selected_index = (selected_index + 1).min(2),
                    KeyCode::Enter => return Ok(index_to_language(selected_index)),
                    _ => {}
                }
            }
        }
    }
}

fn index_to_language(index: usize) -> Language {
    match index {
        0 => Language::En,
        1 => Language::Fr,
        _ => Language::Ja,
    }
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
                handle_key(key, &mut state);
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

fn handle_key(key: KeyEvent, state: &mut AppState) {
    if state.command.is_some() {
        match key.code {
            KeyCode::Esc => state.apply(Action::ExitCommandMode),
            KeyCode::Enter => state.apply(Action::SubmitCommand),
            KeyCode::Backspace => state.apply(Action::CommandBackspace),
            KeyCode::Char(ch) if is_printable(ch) && key.modifiers.is_empty() => {
                state.apply(Action::CommandAppend(ch));
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('h') => state.apply(Action::FocusLeft),
        KeyCode::Char('j') => state.apply(Action::FocusDown),
        KeyCode::Char('k') => state.apply(Action::FocusUp),
        KeyCode::Char('l') => state.apply(Action::FocusRight),
        KeyCode::Tab => state.apply(Action::RotateFocus),
        KeyCode::Enter => state.apply(Action::EnterFullscreen),
        KeyCode::Esc => state.apply(Action::ExitFullscreen),
        KeyCode::Char(':') => state.apply(Action::EnterCommandMode),
        _ => {}
    }
}

fn is_printable(ch: char) -> bool {
    !ch.is_control()
}
