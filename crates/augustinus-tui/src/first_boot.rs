use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme::Theme;

const OPTIONS: &[&str] = &["English", "Français", "日本語"];

pub fn render(frame: &mut Frame<'_>, selected_index: usize) {
    let theme = Theme::arctic();
    let area = frame.area();
    frame.render_widget(Block::default().style(theme.base()), area);

    let [title_area, list_area, hint_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .margin(2)
        .areas(area);

    let title = Paragraph::new("Select language")
        .alignment(Alignment::Center)
        .style(theme.base().fg(theme.fg).bold());
    frame.render_widget(title, title_area);

    let lines: Vec<Line<'static>> = OPTIONS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == selected_index {
                Line::from(format!("> {label}")).style(theme.base().fg(theme.fg))
            } else {
                Line::from(format!("  {label}")).style(theme.base().fg(theme.accent))
            }
        })
        .collect();

    let list = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("First Boot")
                .style(theme.base().fg(theme.accent)),
        )
        .alignment(Alignment::Left)
        .style(theme.base());
    frame.render_widget(list, list_area);

    let hint = Paragraph::new("j/k to move, Enter to confirm (↑/↓ also works)")
        .alignment(Alignment::Center)
        .style(theme.base().fg(theme.accent));
    frame.render_widget(hint, hint_area);
}
