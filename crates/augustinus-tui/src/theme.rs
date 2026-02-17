use ratatui::style::{Color, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
}

impl Theme {
    pub fn arctic() -> Self {
        Self {
            bg: Color::Rgb(4, 18, 36),
            fg: Color::Rgb(235, 245, 255),
            accent: Color::Rgb(120, 220, 255),
            border_focused: Color::Rgb(235, 245, 255),
            border_unfocused: Color::Rgb(120, 220, 255),
        }
    }

    pub fn base(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }
}

