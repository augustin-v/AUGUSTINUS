use std::time::Duration;

use augustinus_app::{MotivationState, Tone};

#[test]
fn rotates_every_15_seconds_without_repeating_consecutively() {
    let mut m = MotivationState::new(Duration::from_secs(5));
    let first = m.quote();
    m.tick(Duration::from_secs(15));
    let second = m.quote();
    assert_ne!(first, second);
}

#[test]
fn idle_switches_to_brutal_and_activity_restores_default() {
    let mut m = MotivationState::new(Duration::from_secs(5));
    assert_eq!(m.tone(), Tone::Encouraging);
    m.tick(Duration::from_secs(5));
    assert_eq!(m.tone(), Tone::Brutal);
    m.on_activity();
    assert_eq!(m.tone(), Tone::Encouraging);
}

