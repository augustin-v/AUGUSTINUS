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
    state: &AppState,
) {
    let focus_seconds = state.focus.focus_seconds_today();
    let streak_days = state.focus.streak_days();
    let loc_line = match state.loc_delta {
        Some(delta) => format!("LOC (git diff): +{} / -{}", delta.added, delta.removed),
        None => "LOC (git diff): N/A".to_string(),
    };
    let text = Text::from(vec![
        Line::from("STATS"),
        Line::from(""),
        Line::from(format!("Streak days: {streak_days}")),
        Line::from(format!("Focus today: {focus_seconds}s")),
        Line::from(loc_line),
    ]);
    let widget = Paragraph::new(text)
        .block(block)
        .style(theme.base())
        .alignment(Alignment::Left);
    frame.render_widget(widget, area);
}
