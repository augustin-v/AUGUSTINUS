use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::theme::Theme;
use crate::widgets::big_text::BigText;
use augustinus_app::{particles::ParticleKind, AppState, Tone, DAILY_FOCUS_GOAL_SECS};

pub fn render(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
) {
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Keep animation state in the app; renderer informs it of the current pane size.
    // This avoids tying particle coordinates to terminal size outside of MotivationState.
    state.motivation.set_particle_bounds(inner.width, inner.height);
    state.motivation.set_ticker_width(inner.width);
    render_particles(frame, state, inner, theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(7),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

    render_header(frame, state, chunks[0], theme);
    render_banner(frame, state, chunks[1], theme);
    render_quote(frame, state, chunks[2], theme);
    render_stats(frame, state, chunks[3], theme);
    render_ticker(frame, state, chunks[4], theme);
}

fn render_particles(frame: &mut Frame<'_>, state: &AppState, inner: ratatui::layout::Rect, theme: &Theme) {
    let buf = frame.buffer_mut();
    for p in state.motivation.particles.points() {
        let gx = inner.x.saturating_add(p.x).min(inner.right().saturating_sub(1));
        let gy = inner.y.saturating_add(p.y).min(inner.bottom().saturating_sub(1));
        if gx < inner.right() && gy < inner.bottom() {
            if let Some(cell) = buf.cell_mut((gx, gy)) {
                cell.set_char(p.ch);
                let style = match p.kind {
                    ParticleKind::Background => theme.base().fg(theme.accent).add_modifier(Modifier::DIM),
                    ParticleKind::Burst => theme.base().fg(theme.border_focused).add_modifier(Modifier::BOLD),
                };
                cell.set_style(style);
            }
        }
    }
}

fn render_header(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect, theme: &Theme) {
    let tone = state.motivation.tone();
    let tone_label = match tone {
        Tone::Brutal => "BRUTAL",
        Tone::Encouraging => "ENCOURAGING",
        Tone::Emperor => "EMPEROR",
    };

    let is_idle = state.motivation.idle.is_idle();
    let idle_style = if is_idle {
        theme
            .base()
            .fg(theme.bg)
            .bg(theme.border_focused)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.base().fg(theme.accent)
    };

    let focus_active = state.focus.is_active();
    let focus_style = if focus_active {
        theme
            .base()
            .fg(theme.bg)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.base().fg(theme.fg).add_modifier(Modifier::DIM)
    };

    let header_line = Line::from(vec![
        Span::styled(tone_label, theme.base().fg(theme.accent)),
        Span::raw(" "),
        Span::styled(
            if is_idle { "• IDLE" } else { "• LIVE" },
            idle_style,
        ),
        Span::raw("  "),
        Span::styled(
            if focus_active { "FOCUS ACTIVE" } else { "FOCUS READY" },
            focus_style,
        ),
    ]);

    let mut sep = "-".repeat(area.width as usize);
    if area.width > 0 {
        let pos = (state.motivation.ticker.offset_cols() as usize) % (area.width as usize);
        sep.replace_range(pos..=pos, "*");
    }

    let text = Text::from(vec![header_line, Line::styled(sep, theme.base().fg(theme.accent))]);
    frame.render_widget(
        Paragraph::new(text).style(theme.base()).alignment(Alignment::Left),
        area,
    );
}

fn render_banner(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect, theme: &Theme) {
    let streak_days = state.focus.streak_days();
    let banner = if state.motivation.idle.is_idle() {
        "WAKE UP".to_string()
    } else if state.focus.is_active() {
        "LOCK IN".to_string()
    } else if streak_days > 0 {
        format!("STREAK {streak_days}")
    } else {
        "DEEP WORK".to_string()
    };

    let intensity = state.motivation.pulse.intensity_0_to_255();
    let mut style = if state.motivation.idle.is_idle() {
        theme.base().fg(theme.border_focused)
    } else {
        theme.base().fg(theme.accent)
    };
    if intensity > 210 {
        style = style.add_modifier(Modifier::BOLD);
    } else if intensity < 60 {
        style = style.add_modifier(Modifier::DIM);
    }
    if state.motivation.cool_down_remaining() > std::time::Duration::ZERO {
        style = style.add_modifier(Modifier::DIM);
    }
    if state.motivation.wake_pulse_remaining() > std::time::Duration::ZERO {
        style = style.add_modifier(Modifier::BOLD);
    }

    let big = BigText::new(&banner);
    let lines = big
        .lines()
        .into_iter()
        .map(|l| Line::styled(l, style))
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .style(theme.base())
            .alignment(Alignment::Center),
        area,
    );
}

fn render_quote(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect, theme: &Theme) {
    let card = Block::default()
        .borders(Borders::ALL)
        .title("NOW")
        .style(theme.base().fg(theme.accent));

    let text = state.motivation.typewriter.visible_text();
    frame.render_widget(
        Paragraph::new(text)
            .block(card)
            .style(theme.base())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_stats(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect, theme: &Theme) {
    let focus_seconds = state.focus.focus_seconds_today();
    let streak_days = state.focus.streak_days();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(area);

    let focus_hms = format_hms(focus_seconds);
    let goal_hms = format_hms(DAILY_FOCUS_GOAL_SECS);

    let stats_text = Text::from(vec![
        Line::from(vec![
            Span::styled("STREAK ", theme.base().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{streak_days} day(s)"),
                theme.base().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("TODAY ", theme.base().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{focus_hms} / {goal_hms}"),
                theme.base().fg(theme.accent),
            ),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(stats_text).style(theme.base()).alignment(Alignment::Left),
        rows[0],
    );

    let ratio = if DAILY_FOCUS_GOAL_SECS == 0 {
        0.0
    } else {
        (focus_seconds as f64 / DAILY_FOCUS_GOAL_SECS as f64).min(1.0)
    };
    let gauge = Gauge::default()
        .ratio(ratio)
        .style(theme.base().fg(theme.accent))
        .gauge_style(theme.base().fg(theme.accent).add_modifier(Modifier::BOLD))
        .label(format!("{:.0}%", ratio * 100.0));
    frame.render_widget(gauge, rows[1]);
}

fn render_ticker(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(state.motivation.ticker_window())
            .style(theme.base().fg(theme.accent))
            .alignment(Alignment::Left),
        area,
    );
}

fn format_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
