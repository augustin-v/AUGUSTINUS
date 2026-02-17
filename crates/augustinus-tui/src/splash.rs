use std::time::Duration;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;

const ART: &[&str] = &[
    r"    ___    __  _______  ________________  _   ____  ______",
    r"   /   |  / / / / ___/ / / / ___/_  __/ / / / / / / / __ \\",
    r"  / /| | / / / /\__ \ / / / /__  / / / / / / / /_/ / / / /",
    r" / ___ |/ /_/ /___/ // /_/\___/ /_/ / /_/ / / __  / /_/ / ",
    r"/_/  |_\____//____/ \____/____/     \____/ /_/ /_/\____/  ",
];

pub fn render(frame: &mut Frame<'_>, elapsed: Duration) {
    let theme = Theme::arctic();
    let area = frame.area();
    let block = Block::default().style(theme.base());
    frame.render_widget(block, area);

    let shimmer = ((elapsed.as_millis() / 120) % 2) == 0;
    let fg = if shimmer { theme.fg } else { theme.accent };

    let text = Text::from(
        ART.iter()
            .enumerate()
            .map(|(i, line)| Line::styled(*line, line_style(fg, i)))
            .collect::<Vec<_>>(),
    );

    let para = Paragraph::new(text)
        .style(theme.base())
        .alignment(Alignment::Center);

    frame.render_widget(para, centered_rect(area, ART.len() as u16 + 2, 80));
}

fn line_style(fg: ratatui::style::Color, index: usize) -> Style {
    let mut style = Style::default().fg(fg);
    if index == 0 {
        style = style.bold();
    }
    style
}

fn centered_rect(area: Rect, height: u16, width: u16) -> Rect {
    let x = area.x.saturating_add((area.width.saturating_sub(width)) / 2);
    let y = area.y.saturating_add((area.height.saturating_sub(height)) / 2);
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

