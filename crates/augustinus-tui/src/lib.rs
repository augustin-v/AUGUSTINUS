mod layout;
mod panes;
mod theme;

use augustinus_app::AppState;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    layout::render_root(frame, state);
}

