#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    RotateFocus,
    EnterFullscreen,
    ExitFullscreen,
    EnterCommandMode,
    ExitCommandMode,
}

