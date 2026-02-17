use augustinus_app::AppState;
use ratatui::{
    layout::{Position, Rect},
    prelude::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::theme::Theme;

pub fn render(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let Some(buffer) = state.command.as_deref() else {
        return;
    };

    let area = frame.area();
    if area.width < 10 || area.height < 7 {
        return;
    }

    let max_width = area.width.saturating_sub(4);
    let prompt_width = 1 + UnicodeWidthStr::width(buffer);
    let mut overlay_width = (prompt_width as u16).saturating_add(4);
    overlay_width = overlay_width.max(30).min(max_width.max(30).min(area.width));
    let overlay_height = 5u16;

    let overlay = centered_rect(area, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay);

    let block = Block::default()
        .title("COMMAND")
        .borders(Borders::ALL)
        .style(theme.base())
        .border_style(theme.base().fg(theme.accent).bold());

    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let max_buffer_width = inner.width.saturating_sub(1) as usize;
    let visible_buffer = tail_by_display_width(buffer, max_buffer_width);

    let prompt = Line::from(vec![
        Span::styled(":", theme.base().fg(theme.accent).bold()),
        Span::styled(visible_buffer.as_str(), theme.base().fg(theme.fg)),
    ]);
    frame.render_widget(
        Paragraph::new(prompt),
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );

    let hint = Line::from(vec![
        Span::styled("Enter", theme.base().fg(theme.accent).bold()),
        Span::styled(": run  ", theme.base().fg(theme.accent)),
        Span::styled("Esc", theme.base().fg(theme.accent).bold()),
        Span::styled(": cancel", theme.base().fg(theme.accent)),
    ]);
    frame.render_widget(
        Paragraph::new(hint),
        Rect {
            x: inner.x,
            y: inner.y.saturating_add(1),
            width: inner.width,
            height: 1,
        },
    );

    let visible_width = UnicodeWidthStr::width(visible_buffer.as_str()) as u16;
    let cursor_x = inner
        .x
        .saturating_add(1)
        .saturating_add(visible_width)
        .min(inner.x.saturating_add(inner.width.saturating_sub(1)));
    frame.set_cursor_position(Position {
        x: cursor_x,
        y: inner.y,
    });
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(width) / 2),
        y: area.y + (area.height.saturating_sub(height) / 2),
        width,
        height,
    }
}

fn tail_by_display_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }

    let mut width = 0usize;
    let mut chars = Vec::new();
    for ch in s.chars().rev() {
        let ch_width = ch.width().unwrap_or(0);
        if width.saturating_add(ch_width) > max_width {
            break;
        }
        width += ch_width;
        chars.push(ch);
    }
    chars.into_iter().rev().collect()
}
