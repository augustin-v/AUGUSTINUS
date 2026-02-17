use std::time::Duration;

use augustinus_app::IdleTracker;

#[test]
fn becomes_idle_after_threshold() {
    let mut t = IdleTracker::new(Duration::from_secs(5));
    t.on_activity();
    t.advance(Duration::from_secs(4));
    assert!(!t.is_idle());
    t.advance(Duration::from_secs(2));
    assert!(t.is_idle());
}

