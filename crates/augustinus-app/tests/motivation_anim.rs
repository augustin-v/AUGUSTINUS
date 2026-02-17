use std::time::Duration;

use augustinus_app::motivation_anim::{BannerPulse, QuoteTypewriter, Ticker};

#[test]
fn banner_pulse_cycles() {
    let mut p = BannerPulse::new(Duration::from_millis(1000));
    let a0 = p.intensity_0_to_255();
    p.tick(Duration::from_millis(250));
    let a1 = p.intensity_0_to_255();
    assert_ne!(a0, a1);
}

#[test]
fn typewriter_reveals_over_time() {
    let mut t = QuoteTypewriter::new(10);
    t.set_text("hello");
    assert_eq!(t.visible_len(), 0);
    t.tick(Duration::from_millis(100));
    assert!(t.visible_len() > 0);
    t.tick(Duration::from_secs(5));
    assert_eq!(t.visible_len(), 5);
}

#[test]
fn ticker_advances_offset() {
    let mut t = Ticker::new(10);
    t.set_text("abcdef");
    let o0 = t.offset_cols();
    t.tick(Duration::from_secs(1));
    assert!(t.offset_cols() > o0);
}
