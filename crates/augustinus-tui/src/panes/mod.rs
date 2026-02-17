mod agents;
mod general;
mod motivation;
mod stats;

use augustinus_app::PaneId;
use ratatui::{widgets::Block, Frame};

use crate::theme::Theme;

pub fn title(id: PaneId) -> &'static str {
    match id {
        PaneId::Motivation => "MOTIVATION",
        PaneId::General => "GENERAL",
        PaneId::Agents => "AI AGENTS",
        PaneId::Stats => "STATS",
    }
}

pub fn render(frame: &mut Frame<'_>, id: PaneId, area: ratatui::layout::Rect, block: Block<'static>, theme: &Theme) {
    match id {
        PaneId::Motivation => motivation::render(frame, area, block, theme),
        PaneId::General => general::render(frame, area, block, theme),
        PaneId::Agents => agents::render(frame, area, block, theme),
        PaneId::Stats => stats::render(frame, area, block, theme),
    }
}

