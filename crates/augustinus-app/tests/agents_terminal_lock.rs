use augustinus_app::{Action, AgentsInputMode, AppState, PaneId};

#[test]
fn focus_agents_then_enter_locks() {
    let mut s = AppState::new_for_test();
    s.apply(Action::FocusDown); // Motivation -> Agents
    assert_eq!(s.focused, PaneId::Agents);
    assert_eq!(s.agents_input_mode, AgentsInputMode::PaneControls);
    s.apply(Action::EnterAgentsTerminalMode);
    assert_eq!(s.agents_input_mode, AgentsInputMode::CodexLocked);
}

#[test]
fn leaving_agents_unlocks() {
    let mut s = AppState::new_for_test();
    s.focused = PaneId::Agents;
    s.agents_input_mode = AgentsInputMode::CodexLocked;
    s.apply(Action::FocusUp); // Agents -> Motivation
    assert_eq!(s.focused, PaneId::Motivation);
    assert_eq!(s.agents_input_mode, AgentsInputMode::PaneControls);
}

