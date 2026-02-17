use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleField {
    rng: Lcg,
    width: u16,
    height: u16,
    particles: Vec<Particle>,
    bursts: Vec<BurstParticle>,
    render_cache: Vec<(u16, u16, char)>,
}

impl ParticleField {
    pub fn new(seed: Seed, width: u16, height: u16, count: usize) -> Self {
        let mut rng = Lcg::new(seed.0);
        let width = width.max(1);
        let height = height.max(1);
        let mut particles = Vec::with_capacity(count);
        for _ in 0..count {
            particles.push(Particle::new(&mut rng, width, height));
        }

        let mut this = Self {
            rng,
            width,
            height,
            particles,
            bursts: Vec::new(),
            render_cache: Vec::with_capacity(count),
        };
        this.rebuild_cache();
        this
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        for p in &mut self.particles {
            p.clamp(width, height);
        }
        for b in &mut self.bursts {
            b.clamp(width, height);
        }
        self.rebuild_cache();
    }

    pub fn tick(&mut self, dt: Duration) {
        for p in &mut self.particles {
            p.tick(dt, &mut self.rng, self.width, self.height);
        }

        let dt_ms = dt.as_millis() as u64;
        for b in &mut self.bursts {
            b.tick(dt, self.width, self.height);
            b.ttl_ms = b.ttl_ms.saturating_sub(dt_ms);
        }
        self.bursts.retain(|b| b.ttl_ms > 0);

        self.rebuild_cache();
    }

    pub fn trigger_burst(&mut self, count: usize, ttl: Duration) {
        let ttl_ms = ttl.as_millis().min(u64::MAX as u128) as u64;
        if ttl_ms == 0 || count == 0 {
            return;
        }

        self.bursts.reserve(count);
        for _ in 0..count {
            self.bursts.push(BurstParticle::new(
                &mut self.rng,
                self.width,
                self.height,
                ttl_ms,
            ));
        }

        self.rebuild_cache();
    }

    pub fn points(&self) -> &[(u16, u16, char)] {
        &self.render_cache
    }

    pub fn snapshot(&self) -> Vec<(u16, u16, char)> {
        self.render_cache.clone()
    }

    fn rebuild_cache(&mut self) {
        self.render_cache.clear();
        self.render_cache
            .reserve(self.particles.len().saturating_add(self.bursts.len()));
        for p in &self.particles {
            self.render_cache.push((p.x, p.y_cell(self.height), p.glyph));
        }
        for b in &self.bursts {
            self.render_cache
                .push((b.x_cell(self.width), b.y_cell(self.height), b.glyph));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Particle {
    x: u16,
    y_fp: u32,
    vy_fp: u32,
    glyph: char,
}

impl Particle {
    fn new(rng: &mut Lcg, width: u16, height: u16) -> Self {
        let x = rng.next_u16(width);
        let y_cell = rng.next_u16(height) as u32;
        let y_fp = y_cell << 16;
        let vy_fp = (70_000u32).saturating_add(rng.next_u32() % 90_000); // ~1.1..2.4 cells/sec
        let glyph = match rng.next_u32() % 4 {
            0 => '.',
            1 => '*',
            2 => 'o',
            _ => '+',
        };
        Self {
            x,
            y_fp,
            vy_fp,
            glyph,
        }
    }

    fn clamp(&mut self, width: u16, height: u16) {
        if width == 0 || height == 0 {
            self.x = 0;
            self.y_fp = 0;
            return;
        }
        self.x %= width;
        let max_y_fp = (height as u32).saturating_sub(1) << 16;
        self.y_fp = self.y_fp.min(max_y_fp);
    }

    fn tick(&mut self, dt: Duration, rng: &mut Lcg, width: u16, height: u16) {
        let height_fp = (height as u32) << 16;
        let delta = mul_fp_per_sec(self.vy_fp, dt);
        self.y_fp = self.y_fp.saturating_add(delta);

        if self.y_fp >= height_fp {
            self.y_fp %= height_fp.max(1);
            self.x = rng.next_u16(width);
        }

        if (rng.next_u32() & 0xFF) == 0 {
            let dir = (rng.next_u32() % 3) as i32 - 1;
            let nx = (self.x as i32).saturating_add(dir);
            self.x = nx.clamp(0, (width.saturating_sub(1)) as i32) as u16;
        }
    }

    fn y_cell(&self, height: u16) -> u16 {
        let height = height.max(1) as u32;
        ((self.y_fp >> 16) % height) as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BurstParticle {
    x_fp: u32,
    y_fp: u32,
    vx_fp: i32,
    vy_fp: i32,
    ttl_ms: u64,
    glyph: char,
}

impl BurstParticle {
    fn new(rng: &mut Lcg, width: u16, height: u16, ttl_ms: u64) -> Self {
        let x_fp = (rng.next_u16(width) as u32) << 16;
        let y_fp = (rng.next_u16(height) as u32) << 16;
        let vx_fp = (rng.next_u32() as i32 % 60_000) - 30_000; // ~-0.45..0.45 cells/sec
        let vy_fp = -((rng.next_u32() as i32 % 120_000) + 40_000); // upward impulse
        let glyph = match rng.next_u32() % 3 {
            0 => '*',
            1 => '+',
            _ => '#',
        };
        Self {
            x_fp,
            y_fp,
            vx_fp,
            vy_fp,
            ttl_ms,
            glyph,
        }
    }

    fn clamp(&mut self, width: u16, height: u16) {
        let width_fp = (width.max(1) as u32) << 16;
        let height_fp = (height.max(1) as u32) << 16;
        self.x_fp %= width_fp.max(1);
        self.y_fp %= height_fp.max(1);
    }

    fn tick(&mut self, dt: Duration, width: u16, height: u16) {
        let dx = mul_fp_per_sec_i32(self.vx_fp, dt);
        let dy = mul_fp_per_sec_i32(self.vy_fp, dt);

        self.x_fp = self.x_fp.saturating_add_signed(dx);
        self.y_fp = self.y_fp.saturating_add_signed(dy);

        let width_fp = (width.max(1) as u32) << 16;
        let height_fp = (height.max(1) as u32) << 16;
        if self.x_fp >= width_fp {
            self.x_fp %= width_fp.max(1);
        }
        if self.y_fp >= height_fp {
            self.y_fp %= height_fp.max(1);
        }
    }

    fn x_cell(&self, width: u16) -> u16 {
        let width = width.max(1) as u32;
        ((self.x_fp >> 16) % width) as u16
    }

    fn y_cell(&self, height: u16) -> u16 {
        let height = height.max(1) as u32;
        ((self.y_fp >> 16) % height) as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_u16(&mut self, upper_exclusive: u16) -> u16 {
        let upper = upper_exclusive.max(1) as u32;
        (self.next_u32() % upper) as u16
    }
}

fn mul_fp_per_sec(v_fp: u32, dt: Duration) -> u32 {
    let nanos = dt.as_nanos();
    ((v_fp as u128).saturating_mul(nanos) / 1_000_000_000) as u32
}

fn mul_fp_per_sec_i32(v_fp: i32, dt: Duration) -> i32 {
    let nanos = dt.as_nanos() as i128;
    let v = v_fp as i128;
    ((v.saturating_mul(nanos)) / 1_000_000_000) as i32
}
