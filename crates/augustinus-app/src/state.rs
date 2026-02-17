use crate::{Action, AgentsInputMode, GeneralInputMode, PaneId};
use crate::FocusState;
use crate::{motivation::DEFAULT_IDLE_THRESHOLD, MotivationState};
use crate::LocDelta;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub focused: PaneId,
    pub fullscreen: Option<PaneId>,
    pub command: Option<String>,
    pub last_command: Option<String>,
    pub motivation: MotivationState,
    pub focus: FocusState,
    pub general_screen: String,
    pub general_input_mode: GeneralInputMode,
    pub agents_screen: String,
    pub agents_input_mode: AgentsInputMode,
    pub loc_delta: Option<LocDelta>,
}

impl AppState {
    pub fn new_for_test() -> Self {
        Self {
            focused: PaneId::Motivation,
            fullscreen: None,
            command: None,
            last_command: None,
            motivation: MotivationState::new(DEFAULT_IDLE_THRESHOLD),
            focus: FocusState::new_for_test(),
            general_screen: String::new(),
            general_input_mode: GeneralInputMode::AppControls,
            agents_screen: String::new(),
            agents_input_mode: AgentsInputMode::PaneControls,
            loc_delta: None,
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::FocusLeft => {
                self.focused = focus_left(self.focused);
                self.general_input_mode = GeneralInputMode::AppControls;
                if self.focused != PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
            Action::FocusRight => {
                self.focused = focus_right(self.focused);
                self.general_input_mode = GeneralInputMode::AppControls;
                if self.focused != PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
            Action::FocusUp => {
                self.focused = focus_up(self.focused);
                self.general_input_mode = GeneralInputMode::AppControls;
                if self.focused != PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
            Action::FocusDown => {
                self.focused = focus_down(self.focused);
                self.general_input_mode = GeneralInputMode::AppControls;
                if self.focused != PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
            Action::RotateFocus => {
                self.focused = self.focused.next();
                self.general_input_mode = GeneralInputMode::AppControls;
                if self.focused != PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
            Action::EnterGeneralTerminalMode => {
                if self.focused == PaneId::General {
                    self.general_input_mode = GeneralInputMode::TerminalLocked;
                }
            }
            Action::ExitGeneralTerminalMode => {
                if self.focused == PaneId::General {
                    self.general_input_mode = GeneralInputMode::AppControls;
                }
            }
            Action::EnterAgentsTerminalMode => {
                if self.focused == PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::CodexLocked;
                }
            }
            Action::ExitAgentsTerminalMode => {
                if self.focused == PaneId::Agents {
                    self.agents_input_mode = AgentsInputMode::PaneControls;
                }
            }
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

    pub fn on_activity(&mut self) {
        self.motivation.on_activity();
    }

    pub fn tick(&mut self, dt: std::time::Duration) {
        self.motivation.tick(dt);
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
