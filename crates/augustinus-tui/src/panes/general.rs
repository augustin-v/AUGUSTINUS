use ratatui::{
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;
use augustinus_app::AppState;

pub fn render(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
    state: &AppState,
) {
    let mut lines = Vec::new();
    lines.push(Line::from("Ctrl-Space then h/j/k/l/Tab/:/Esc/Enter for app controls").style(
        theme.base().fg(theme.accent),
    ));
    lines.push(Line::from(""));

    let contents = if state.general_screen.is_empty() {
        "Starting shell…".to_string()
    } else {
        state.general_screen.clone()
    };

    let max_lines = area.height.saturating_sub(4) as usize;
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
