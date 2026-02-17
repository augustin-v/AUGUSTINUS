use augustinus_app::{Action, AppState, PaneId};

#[test]
fn hjkl_moves_focus_in_grid() {
    let mut s = AppState::new_for_test();
    assert_eq!(s.focused, PaneId::Motivation);
    s.apply(Action::FocusRight);
    assert_eq!(s.focused, PaneId::General);
    s.apply(Action::FocusDown);
    assert_eq!(s.focused, PaneId::Stats);
}

