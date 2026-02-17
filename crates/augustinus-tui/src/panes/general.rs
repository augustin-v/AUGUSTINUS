use ratatui::{
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: ratatui::layout::Rect, block: Block<'static>, theme: &Theme) {
    let text = Text::from(vec![
        Line::from("GENERAL pane (PTY later)"),
        Line::from(""),
        Line::from("Placeholder shell view."),
    ]);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}

