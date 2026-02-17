use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use anyhow::Context;
use augustinus_app::{Action, AppState, GeneralInputMode, LocDelta, PaneId};
use augustinus_pty::PtySession;
use augustinus_store::config::{AppConfig, Language};
use augustinus_store::db::Store;
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
        let mut config = AppConfig::load_or_none()
            .map_err(anyhow_to_io)?
            .unwrap_or(AppConfig {
            language: Language::En,
            shell: "/bin/bash".to_string(),
            git_repo: None,
            agents_cmd: None,
        });
        let chosen_language = run_language_picker(&mut terminal, config.language)?;
        config.language = chosen_language;
        let _ = config.save().map_err(anyhow_to_io)?;
        run_app(&mut terminal, &config)
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

fn run_language_picker(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    default: Language,
) -> io::Result<Language> {
    let mut selected_index: usize = language_to_index(default);
    let tick_rate = Duration::from_millis(33);

    loop {
        terminal.draw(|frame| {
            augustinus_tui::render_first_boot(frame, selected_index);
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if should_quit(key) {
                    return Ok(default);
                }
                match key.code {
                    KeyCode::Char('k') | KeyCode::Up => {
                        selected_index = selected_index.saturating_sub(1)
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        selected_index = (selected_index + 1).min(2)
                    }
                    KeyCode::Enter => return Ok(index_to_language(selected_index)),
                    _ => {}
                }
            }
        }
    }
}

fn language_to_index(language: Language) -> usize {
    match language {
        Language::En => 0,
        Language::Fr => 1,
        Language::Ja => 2,
    }
}

fn index_to_language(index: usize) -> Language {
    match index {
        0 => Language::En,
        1 => Language::Fr,
        _ => Language::Ja,
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, config: &AppConfig) -> io::Result<()> {
    let mut state = AppState::new_for_test();
    let store = init_store_and_load_stats(&mut state)?;
    let mut git_poll_elapsed = Duration::from_secs(30);

    let size = terminal.size()?;
    let (general_cols, general_rows) = general_pty_size(&state, size.width, size.height);
    let (agents_cols, agents_rows) = pane_pty_size(&state, size.width, size.height, PaneId::Agents);

    let mut pty = PtySession::spawn(&config.shell, general_cols, general_rows).map_err(anyhow_to_io)?;
    let mut last_general_cols = general_cols;
    let mut last_general_rows = general_rows;

    let mut agents_pty = match config
        .agents_cmd
        .as_ref()
        .and_then(|cmd| cmd.split_first())
    {
        Some((program, args)) => {
            let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            PtySession::spawn_command(program, &args, agents_cols, agents_rows)
        }
        None => PtySession::spawn_command("codex", &[], agents_cols, agents_rows),
    }
    .or_else(|_| {
        let mut fallback =
            PtySession::spawn(&config.shell, agents_cols, agents_rows).context("spawn agents fallback")?;
        let _ = fallback.send_paste("echo 'codex not found; install it, then restart'\n");
        Ok(fallback)
    })
    .map_err(anyhow_to_io)?;

    let mut last_agents_cols = agents_cols;
    let mut last_agents_rows = agents_rows;
    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            augustinus_tui::render(frame, &mut state);
        })?;

        pty.poll();
        state.general_screen = pty.snapshot().contents;

        agents_pty.poll();
        state.agents_screen = agents_pty.snapshot().contents;

        let size = terminal.size()?;
        let (general_cols, general_rows) = general_pty_size(&state, size.width, size.height);
        if general_cols != last_general_cols || general_rows != last_general_rows {
            let _ = pty.resize(general_cols, general_rows);
            last_general_cols = general_cols;
            last_general_rows = general_rows;
        }

        let (agents_cols, agents_rows) = pane_pty_size(&state, size.width, size.height, PaneId::Agents);
        if agents_cols != last_agents_cols || agents_rows != last_agents_rows {
            let _ = agents_pty.resize(agents_cols, agents_rows);
            last_agents_cols = agents_cols;
            last_agents_rows = agents_rows;
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                state.on_activity();
                if handle_key(key, &mut state, &mut pty) {
                    break;
                }
                if let Some(cmd) = state.last_command.take() {
                    handle_command(&cmd, &mut state, &store)?;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let dt = last_tick.elapsed();
            state.tick(dt);
            git_poll_elapsed = git_poll_elapsed.saturating_add(dt);
            if git_poll_elapsed >= Duration::from_secs(30) {
                git_poll_elapsed = Duration::ZERO;
                if let Some(repo) = config.git_repo.as_deref() {
                    state.loc_delta = compute_loc_delta(repo);
                } else {
                    state.loc_delta = None;
                }
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn compute_loc_delta(repo_path: &str) -> Option<LocDelta> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("diff")
        .arg("--numstat")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(LocDelta::parse_git_numstat(&stdout))
}

fn general_pty_size(state: &AppState, term_cols: u16, term_rows: u16) -> (u16, u16) {
    pane_pty_size(state, term_cols, term_rows, PaneId::General)
}

fn pane_pty_size(state: &AppState, term_cols: u16, term_rows: u16, pane_id: PaneId) -> (u16, u16) {
    let (cols, rows) = match state.fullscreen {
        Some(id) if id == pane_id => (term_cols, term_rows),
        _ => (term_cols / 2, term_rows / 2),
    };
    (cols.saturating_sub(2).max(1), rows.saturating_sub(2).max(1))
}

fn init_store_and_load_stats(state: &mut AppState) -> io::Result<Store> {
    let db_path = Store::default_db_path().map_err(anyhow_to_io)?;
    let store = Store::open(db_path).map_err(anyhow_to_io)?;

    let today = chrono::Local::now().date_naive();
    let focus_seconds_today = store
        .focus_seconds_for_day(today)
        .map_err(anyhow_to_io)?
        .max(0) as u64;
    let streak = store.streak_days_ending_today().map_err(anyhow_to_io)?;
    state.focus.set_focus_seconds_today(focus_seconds_today);
    state.focus.set_streak_days(streak);

    Ok(store)
}

fn handle_command(cmd: &str, state: &mut AppState, store: &Store) -> io::Result<()> {
    let cmd = cmd.trim();

    if let Some(rest) = cmd.strip_prefix("focus ") {
        let arg = rest.trim();
        match arg {
            "start" => {
                store.insert_event("focus_start", "{}").map_err(anyhow_to_io)?;
                if state.focus.start(Instant::now()) {
                    state.motivation.on_focus_start();
                }
            }
            "stop" => {
                if let Some(elapsed) = state.focus.stop(Instant::now()) {
                    let secs = elapsed.as_secs().min(i64::MAX as u64) as i64;
                    store
                        .insert_event("focus_stop", &format!(r#"{{"seconds":{secs}}}"#))
                        .map_err(anyhow_to_io)?;
                    store.add_focus_seconds_today(secs).map_err(anyhow_to_io)?;
                    state.focus.add_focus_seconds_today(secs.max(0) as u64);
                    let streak = store.streak_days_ending_today().map_err(anyhow_to_io)?;
                    state.focus.set_streak_days(streak);
                    state.motivation.on_focus_stop();
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn should_quit(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn handle_key(
    key: KeyEvent,
    state: &mut AppState,
    pty: &mut PtySession,
) -> bool {
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
        return false;
    }

    if state.focused == PaneId::General
        && state.general_input_mode == GeneralInputMode::TerminalLocked
    {
        if key.code == KeyCode::Esc {
            state.apply(Action::ExitGeneralTerminalMode);
            return false;
        }
        let _ = pty.send_key(key);
        return false;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return true;
    }

    match key.code {
        KeyCode::Char('h') => state.apply(Action::FocusLeft),
        KeyCode::Char('j') => state.apply(Action::FocusDown),
        KeyCode::Char('k') => state.apply(Action::FocusUp),
        KeyCode::Char('l') => state.apply(Action::FocusRight),
        KeyCode::Tab => state.apply(Action::RotateFocus),
        KeyCode::Enter => {
            if state.focused == PaneId::General
                && state.general_input_mode == GeneralInputMode::AppControls
            {
                state.apply(Action::EnterGeneralTerminalMode);
                return false;
            }
            state.apply(Action::EnterFullscreen)
        }
        KeyCode::Esc => {
            if state.fullscreen.is_some() {
                state.apply(Action::ExitFullscreen)
            }
        }
        KeyCode::Char(':') => state.apply(Action::EnterCommandMode),
        _ => {}
    }

    false
}

fn is_printable(ch: char) -> bool {
    !ch.is_control()
}
