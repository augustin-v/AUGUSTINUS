use crate::{Action, PaneId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub focused: PaneId,
    pub fullscreen: Option<PaneId>,
    pub command: Option<String>,
    pub last_command: Option<String>,
}

impl AppState {
    pub fn new_for_test() -> Self {
        Self {
            focused: PaneId::Motivation,
            fullscreen: None,
            command: None,
            last_command: None,
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::FocusLeft => self.focused = focus_left(self.focused),
            Action::FocusRight => self.focused = focus_right(self.focused),
            Action::FocusUp => self.focused = focus_up(self.focused),
            Action::FocusDown => self.focused = focus_down(self.focused),
            Action::RotateFocus => self.focused = self.focused.next(),
            Action::EnterFullscreen => self.fullscreen = Some(self.focused),
            Action::ExitFullscreen => self.fullscreen = None,
            Action::EnterCommandMode => self.command = Some(String::new()),
            Action::ExitCommandMode => self.command = None,
            Action::CommandAppend(ch) => {
                if let Some(buffer) = self.command.as_mut() {
                    buffer.push(ch);
                }
            }
            Action::CommandBackspace => {
                if let Some(buffer) = self.command.as_mut() {
                    buffer.pop();
                }
            }
            Action::SubmitCommand => {
                if let Some(buffer) = self.command.take() {
                    self.last_command = Some(buffer);
                }
            }
        }
    }
}

fn focus_left(current: PaneId) -> PaneId {
    match current {
        PaneId::Motivation => PaneId::Motivation,
        PaneId::General => PaneId::Motivation,
        PaneId::Agents => PaneId::Agents,
        PaneId::Stats => PaneId::Agents,
    }
}

fn focus_right(current: PaneId) -> PaneId {
    match current {
        PaneId::Motivation => PaneId::General,
        PaneId::General => PaneId::General,
        PaneId::Agents => PaneId::Stats,
        PaneId::Stats => PaneId::Stats,
    }
}

fn focus_up(current: PaneId) -> PaneId {
    match current {
        PaneId::Motivation => PaneId::Motivation,
        PaneId::General => PaneId::General,
        PaneId::Agents => PaneId::Motivation,
        PaneId::Stats => PaneId::General,
    }
}

fn focus_down(current: PaneId) -> PaneId {
    match current {
        PaneId::Motivation => PaneId::Agents,
        PaneId::General => PaneId::Stats,
        PaneId::Agents => PaneId::Agents,
        PaneId::Stats => PaneId::Stats,
    }
}
