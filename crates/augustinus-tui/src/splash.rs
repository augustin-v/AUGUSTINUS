use std::time::Duration;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;

const ART_STR: &str = include_str!("../../../augustinus_ascii.txt");

pub fn render(frame: &mut Frame<'_>, elapsed: Duration) {
    let theme = Theme::arctic();
    let area = frame.area();
    let block = Block::default().style(theme.base());
    frame.render_widget(block, area);

    let art_lines: Vec<&str> = ART_STR.lines().map(|line| line.trim_end()).collect();
    let height = (art_lines.len() as u16).saturating_add(2);
    let max_line_len = art_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let width = (max_line_len as u16).max(1);

    let shimmer = ((elapsed.as_millis() / 120) % 2) == 0;
    let fg = if shimmer { theme.fg } else { theme.accent };

    let rect = centered_rect(area, height, width);
    let crop_width = rect.width.max(1);
    let text = Text::from(
        art_lines
            .iter()
            .enumerate()
            .map(|(i, line)| Line::styled(center_crop(line, crop_width), line_style(fg, i)))
            .collect::<Vec<_>>(),
    );

    let para = Paragraph::new(text)
        .style(theme.base())
        .alignment(Alignment::Center);

    frame.render_widget(para, rect);
}

fn line_style(fg: ratatui::style::Color, index: usize) -> Style {
    let mut style = Style::default().fg(fg);
    if index == 0 {
        style = style.bold();
    }
    style
}

fn center_crop(line: &str, width: u16) -> String {
    let width = width as usize;
    if width == 0 || line.len() <= width {
        return line.to_string();
    }
    let start = (line.len() - width) / 2;
    line[start..start + width].to_string()
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
