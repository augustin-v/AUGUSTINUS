use ratatui::{
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: ratatui::layout::Rect, block: Block<'static>, theme: &Theme) {
    let text = Text::from(vec![
        Line::from("Motivation pane (MVP stub)"),
        Line::from(""),
        Line::from("Use h/j/k/l or Tab to change focus."),
    ]);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}

