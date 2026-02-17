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
    _state: &mut AppState,
) {
    let text = Text::from(vec![
        Line::from("AI AGENTS pane (MVP stub)"),
        Line::from(""),
        Line::from("Prompt UI comes later."),
    ]);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}
