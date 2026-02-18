use augustinus_app::{AppState, DAILY_FOCUS_GOAL_SECS, PaneId, Tone};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::{theme::Theme, widgets::big_text::BigText};

pub fn render(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
    state: &mut AppState,
) {
    let block = block.style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 28 || inner.height < 10 {
        render_compact(frame, inner, theme, state);
        return;
    }

    let cards_height = inner.height.saturating_sub(3).min(10);

    let top_bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(cards_height), Constraint::Min(0)])
        .split(inner);
    let top = *top_bottom.get(0).unwrap_or(&inner);
    let bottom = *top_bottom.get(1).unwrap_or(&inner);

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(top);
    let c1 = *cards.get(0).unwrap_or(&top);
    let c2 = *cards.get(1).unwrap_or(&top);
    let c3 = *cards.get(2).unwrap_or(&top);

    render_streak_card(frame, c1, theme, state);
    render_focus_card(frame, c2, theme, state);
    render_loc_card(frame, c3, theme, state);
    render_status_table(frame, bottom, theme, state);
}

fn render_compact(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    theme: &Theme,
    state: &AppState,
) {
    let focus_seconds = state.focus.focus_seconds_today();
    let streak_days = state.focus.streak_days();
    let loc_line = match state.loc_delta {
        Some(delta) => format!("LOC: +{} / -{}", delta.added, delta.removed),
        None => "LOC: N/A".to_string(),
    };

    let text = Text::from(vec![
        Line::from(format!("Streak: {streak_days} day(s)")),
        Line::from(format!(
            "Focus: {} / {}",
            format_hms(focus_seconds),
            format_hms(DAILY_FOCUS_GOAL_SECS)
        )),
        Line::from(loc_line),
    ]);

    frame.render_widget(
        Paragraph::new(text)
            .style(theme.base())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_streak_card(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    theme: &Theme,
    state: &AppState,
) {
    let block = Block::default()
        .title("STREAK")
        .borders(Borders::ALL)
        .style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let streak_days = state.focus.streak_days();
    let big = BigText::new(&streak_days.to_string());
    let mut lines: Vec<Line> = big
        .lines()
        .into_iter()
        .map(|s| {
            Line::from(Span::styled(
                s,
                theme
                    .base()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    lines.push(Line::from(Span::styled("days", theme.base().fg(theme.fg))));

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .style(theme.base())
            .alignment(Alignment::Center),
        inner,
    );
}

fn render_focus_card(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    theme: &Theme,
    state: &AppState,
) {
    let block = Block::default()
        .title("FOCUS")
        .borders(Borders::ALL)
        .style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let focus_seconds = state.focus.focus_seconds_today();
    let ratio = if DAILY_FOCUS_GOAL_SECS == 0 {
        0.0
    } else {
        (focus_seconds as f64 / DAILY_FOCUS_GOAL_SECS as f64).min(1.0)
    };

    let top_gauge = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);
    let top = *top_gauge.get(0).unwrap_or(&inner);
    let gauge_area = *top_gauge.get(1).unwrap_or(&inner);

    let top_text = Text::from(vec![
        Line::from(vec![
            Span::styled("TODAY ", theme.base().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(
                format_hms(focus_seconds),
                theme
                    .base()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("GOAL  ", theme.base().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format_hms(DAILY_FOCUS_GOAL_SECS), theme.base().fg(theme.accent)),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(top_text)
            .style(theme.base())
            .alignment(Alignment::Left),
        top,
    );

    let gauge = Gauge::default()
        .ratio(ratio)
        .style(theme.base().fg(theme.accent))
        .gauge_style(theme.base().fg(theme.accent).add_modifier(Modifier::BOLD))
        .label(format!("{:.0}%", ratio * 100.0));
    frame.render_widget(gauge, gauge_area);
}

fn render_loc_card(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    theme: &Theme,
    state: &AppState,
) {
    let block = Block::default()
        .title("LOC")
        .borders(Borders::ALL)
        .style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = match state.loc_delta {
        Some(delta) => vec![
            Line::from(vec![
                Span::styled("+", theme.base().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(
                    delta.added.to_string(),
                    theme
                        .base()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "-",
                    theme
                        .base()
                        .fg(theme.border_focused)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    delta.removed.to_string(),
                    theme
                        .base()
                        .fg(theme.border_focused)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!("net {}", delta.added.saturating_sub(delta.removed)),
                theme.base().fg(theme.fg),
            )),
        ],
        None => vec![Line::from("git diff: N/A")],
    };

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .style(theme.base())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn render_status_table(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    theme: &Theme,
    state: &AppState,
) {
    let block = Block::default()
        .title("STATUS")
        .borders(Borders::ALL)
        .style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tone = match state.motivation.tone() {
        Tone::Brutal => "Brutal",
        Tone::Encouraging => "Encouraging",
        Tone::Emperor => "Emperor",
    };

    let focused = match state.focused {
        PaneId::Motivation => "Motivation",
        PaneId::General => "General",
        PaneId::Agents => "Agents",
        PaneId::Stats => "Stats",
    };

    let fullscreen = state.fullscreen.map(|p| match p {
        PaneId::Motivation => "Motivation",
        PaneId::General => "General",
        PaneId::Agents => "Agents",
        PaneId::Stats => "Stats",
    });

    let rows = vec![
        Row::new(vec![Cell::from("Focused"), Cell::from(focused)]),
        Row::new(vec![Cell::from("Fullscreen"), Cell::from(fullscreen.unwrap_or("No"))]),
        Row::new(vec![
            Cell::from("Focus active"),
            Cell::from(if state.focus.is_active() { "Yes" } else { "No" }),
        ]),
        Row::new(vec![
            Cell::from("Idle"),
            Cell::from(if state.motivation.idle.is_idle() {
                "Yes"
            } else {
                "No"
            }),
        ]),
        Row::new(vec![Cell::from("Tone"), Cell::from(tone)]),
        Row::new(vec![
            Cell::from("Last command"),
            Cell::from(
                state
                    .last_command
                    .clone()
                    .unwrap_or_else(|| "—".to_string()),
            ),
        ]),
    ];

    let table = Table::new(rows, [Constraint::Length(14), Constraint::Min(10)])
        .style(theme.base())
        .column_spacing(1)
        .header(
            Row::new(vec![
                Cell::from(Span::styled(
                    "Key",
                    theme.base().fg(theme.fg).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Value",
                    theme.base().fg(theme.fg).add_modifier(Modifier::BOLD),
                )),
            ])
            .style(theme.base()),
        )
        .row_highlight_style(Style::default());

    frame.render_widget(table, inner);
}

fn format_hms(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
