#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    RotateFocus,
    ToggleGeneralInputMode,
    EnterFullscreen,
    ExitFullscreen,
    EnterCommandMode,
    ExitCommandMode,
    CommandAppend(char),
    CommandBackspace,
    SubmitCommand,
}
