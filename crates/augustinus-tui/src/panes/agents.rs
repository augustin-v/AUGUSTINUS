use ratatui::{
    layout::Alignment,
    layout::Position,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::theme::Theme;
use augustinus_app::{AgentsInputMode, AppState, PaneId};

pub fn render(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    block: Block<'static>,
    theme: &Theme,
    state: &mut AppState,
) {
    let inner = block.inner(area);

    let mut lines = Vec::new();
    if state.agents_input_mode == AgentsInputMode::CodexLocked {
        lines.push(
            Line::from("CODEX LOCKED — Esc to return to pane controls")
                .style(theme.base().fg(theme.accent)),
        );
        lines.push(Line::from(""));
    } else {
        if state.focused == PaneId::Agents {
            lines.push(
                Line::from("Enter: control Codex  Esc: return  h/j/k/l Tab: move focus")
                    .style(theme.base().fg(theme.accent)),
            );
        } else {
            lines.push(
                Line::from("Focus with h/j/k/l; Enter to lock; \":\" commands")
                    .style(theme.base().fg(theme.accent)),
            );
        }
        lines.push(Line::from(""));
    }

    let header_height = lines.len() as u16;

    let contents = if state.agents_screen.is_empty() {
        "Starting codex…".to_string()
    } else {
        state.agents_screen.clone()
    };

    let reserved_height = header_height.saturating_add(2);
    let max_lines = area.height.saturating_sub(reserved_height) as usize;
    let all_lines: Vec<&str> = contents.lines().collect();
    let visible_start = all_lines.len().saturating_sub(max_lines);
    for line in &all_lines[visible_start..] {
        lines.push(Line::from(line.to_string()));
    }

    let text = Text::from(lines);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);

    if state.focused == PaneId::Agents && state.agents_input_mode == AgentsInputMode::CodexLocked {
        let cursor_row = state.agents_cursor_row as usize;
        let cursor_col = state.agents_cursor_col as u16;

        let cursor_row_in_view = if cursor_row >= visible_start {
            (cursor_row - visible_start) as u16
        } else {
            0
        };

        let cursor_x = inner
            .x
            .saturating_add(cursor_col)
            .min(inner.x.saturating_add(inner.width.saturating_sub(1)));
        let cursor_y = inner
            .y
            .saturating_add(header_height)
            .saturating_add(cursor_row_in_view)
            .min(inner.y.saturating_add(inner.height.saturating_sub(1)));

        frame.set_cursor_position(Position {
            x: cursor_x,
            y: cursor_y,
        });
    }
}
