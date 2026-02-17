use ratatui::{
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;
use augustinus_app::{AppState, GeneralInputMode};

pub fn render(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
    state: &AppState,
) {
    let mut lines = Vec::new();
    if state.general_input_mode == GeneralInputMode::TerminalLocked {
        lines.push(
            Line::from("TERMINAL MODE (locked) — Esc to return to app controls")
                .style(theme.base().fg(theme.accent)),
        );
        lines.push(Line::from(""));
    } else {
        lines.push(Line::from("Press Enter to fullscreen; Focus with h/j/k/l; \":\" commands").style(
            theme.base().fg(theme.accent),
        ));
        lines.push(Line::from(""));
    }

    let contents = if state.general_screen.is_empty() {
        "Starting shell…".to_string()
    } else {
        state.general_screen.clone()
    };

    let reserved_height = (lines.len() as u16).saturating_add(2);
    let max_lines = area.height.saturating_sub(reserved_height) as usize;
    let all_lines: Vec<&str> = contents.lines().collect();
    let start = all_lines.len().saturating_sub(max_lines);
    for line in &all_lines[start..] {
        lines.push(Line::from(line.to_string()));
    }

    let text = Text::from(lines);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}
