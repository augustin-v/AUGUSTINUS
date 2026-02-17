use std::time::Duration;

use augustinus_app::particles::{ParticleField, Seed};

#[test]
fn particles_move_down_over_time() {
    let mut f = ParticleField::new(Seed(1), 10, 10, 5);
    let before = f.snapshot();
    f.tick(Duration::from_secs(1));
    let after = f.snapshot();
    assert_ne!(before, after);
}
