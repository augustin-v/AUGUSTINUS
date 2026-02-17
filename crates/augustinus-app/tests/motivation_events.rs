use std::time::Duration;

use augustinus_app::MotivationState;

#[test]
fn focus_start_sets_burst_and_decays() {
    let mut m = MotivationState::new(Duration::from_secs(60));
    m.on_focus_start();
    assert!(m.burst_remaining() > Duration::ZERO);
    let before = m.burst_remaining();
    m.tick(Duration::from_millis(100));
    assert!(m.burst_remaining() < before);
}
