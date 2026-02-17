use augustinus_app::{Action, AppState, GeneralInputMode, PaneId};

#[test]
fn general_starts_in_app_mode() {
    let s = AppState::new_for_test();
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn toggling_general_mode_works() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::General;
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalPassthrough);
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn leaving_general_resets_to_app_mode() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::General;
    s.apply(Action::ToggleGeneralInputMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalPassthrough);
    s.apply(Action::FocusLeft); // General -> Motivation
    assert_eq!(s.focused, PaneId::Motivation);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

