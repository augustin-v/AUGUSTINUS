use augustinus_app::{Action, AppState, GeneralInputMode, PaneId};

#[test]
fn general_starts_in_app_mode() {
    let s = AppState::new_for_test();
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn focusing_general_does_not_lock() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusRight); // Motivation -> General
    assert_eq!(s.focused, PaneId::General);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn leaving_general_resets_to_app_mode() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusRight); // Motivation -> General
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
    s.apply(Action::FocusLeft); // General -> Motivation
    assert_eq!(s.focused, PaneId::Motivation);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn enter_locks_and_exit_unlocks() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusRight); // Motivation -> General
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
    s.apply(Action::EnterGeneralTerminalMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalLocked);
    s.apply(Action::ExitGeneralTerminalMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::AppControls);
}

#[test]
fn enter_general_terminal_mode_sets_terminal_locked() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::General;
    s.general_input_mode = GeneralInputMode::AppControls;
    s.apply(Action::EnterGeneralTerminalMode);
    assert_eq!(s.general_input_mode, GeneralInputMode::TerminalLocked);
}
