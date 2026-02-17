mod first_boot;
mod layout;
mod panes;
mod splash;
mod theme;
pub mod widgets;

use augustinus_app::AppState;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    layout::render_root(frame, state);
}

pub fn render_splash(frame: &mut Frame<'_>, elapsed: std::time::Duration) {
    splash::render(frame, elapsed);
}

pub fn render_first_boot(frame: &mut Frame<'_>, selected_index: usize) {
    first_boot::render(frame, selected_index);
}
