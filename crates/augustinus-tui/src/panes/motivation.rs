use ratatui::{
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;
use augustinus_app::{AppState, Tone};

pub fn render(
    frame: &mut Frame<'_>,
    state: &AppState,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
) {
    let tone = state.motivation.tone();
    let tone_label = match tone {
        Tone::Brutal => "BRUTAL",
        Tone::Encouraging => "ENCOURAGING",
        Tone::Emperor => "EMPEROR",
    };
    let idle_label = if state.motivation.idle.is_idle() {
        " • IDLE"
    } else {
        ""
    };

    let header = format!("{tone_label}{idle_label}");

    let text = Text::from(vec![
        Line::from(header).style(theme.base().fg(theme.accent)),
        Line::from(""),
        Line::from(state.motivation.quote()).style(theme.base().fg(theme.fg)),
    ]);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}
