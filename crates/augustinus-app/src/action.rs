#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    RotateFocus,
    EnterGeneralTerminalMode,
    ExitGeneralTerminalMode,
    EnterFullscreen,
    ExitFullscreen,
    EnterCommandMode,
    ExitCommandMode,
    CommandAppend(char),
    CommandBackspace,
    SubmitCommand,
}
