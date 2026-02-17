use augustinus_app::{AppState, PaneId};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    widgets::{Block, Borders},
    Frame,
};

use crate::{panes, theme::Theme};

pub fn render_root(frame: &mut Frame<'_>, state: &mut AppState) {
    let theme = Theme::arctic();
    frame.render_widget(Block::default().style(theme.base()), frame.area());

    if let Some(fullscreen) = state.fullscreen {
        render_pane(frame, state, fullscreen, frame.area(), &theme);
        return;
    }

    let [top, bottom] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(frame.area());

    let [tl, tr] = split_h(top);
    let [bl, br] = split_h(bottom);

    render_pane(frame, state, PaneId::Motivation, tl, &theme);
    render_pane(frame, state, PaneId::General, tr, &theme);
    render_pane(frame, state, PaneId::Agents, bl, &theme);
    render_pane(frame, state, PaneId::Stats, br, &theme);
}

fn split_h(area: Rect) -> [Rect; 2] {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(area)
}

fn render_pane(frame: &mut Frame<'_>, state: &mut AppState, id: PaneId, area: Rect, theme: &Theme) {
    let focused = state.focused == id;
    let title = panes::title(id);
    let border_color = if focused {
        theme.border_focused
    } else {
        theme.border_unfocused
    };
    let border_style = if focused {
        theme.base().fg(border_color).bold()
    } else {
        theme.base().fg(border_color)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    panes::render(frame, state, id, area, block, theme);
}
