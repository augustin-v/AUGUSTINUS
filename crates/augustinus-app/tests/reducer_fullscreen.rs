use augustinus_app::{Action, AppState, PaneId};

#[test]
fn enter_and_exit_fullscreen() {
    let mut s = AppState::new_for_test();
    s.apply(Action::EnterFullscreen);
    assert_eq!(s.fullscreen, Some(PaneId::Motivation));
    s.apply(Action::ExitFullscreen);
    assert_eq!(s.fullscreen, None);
}

