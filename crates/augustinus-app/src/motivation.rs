use std::time::Duration;

use crate::motivation_anim::{BannerPulse, QuoteTypewriter, Ticker};
use crate::particles::{ParticleField, Seed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Brutal,
    Encouraging,
    Emperor,
}

pub const DEFAULT_ROTATE_EVERY: Duration = Duration::from_secs(15);
pub const DEFAULT_IDLE_THRESHOLD: Duration = Duration::from_secs(60);
pub const BANNER_PULSE_PERIOD: Duration = Duration::from_millis(900);
pub const QUOTE_TYPEWRITER_SPEED_CPS: u32 = 50;
pub const TICKER_SPEED_CPS: u32 = 18;
pub const DAILY_FOCUS_GOAL_SECS: u64 = 2 * 60 * 60;
pub const DEFAULT_TICKER_TEXT: &str = "LOCK IN • NO MERCY • COMPOUND TODAY • STAY DANGEROUS • ONE MORE REP •";
pub const PARTICLE_COUNT: usize = 48;
pub const BURST_TTL: Duration = Duration::from_millis(900);
pub const COOL_DOWN_TTL: Duration = Duration::from_millis(600);
pub const WAKE_PULSE_TTL: Duration = Duration::from_millis(700);

static BRUTAL_QUOTES: &[&str] = &[
    "Your biological clock advances whether you code or not.",
    "Every unfocused hour is stolen from your future children.",
    "Comfort is the tax you pay for mediocrity.",
    "Entropy wins unless you resist it.",
    "Your body is decaying in the background. Work anyway.",
    "You will not get these years back. Stop negotiating.",
    "If you drift today, your future self will pay interest.",
    "The calendar doesn't care about your mood.",
    "A soft day is a vote for a smaller life.",
];

static ENCOURAGING_QUOTES: &[&str] = &[
    "You are building something compounding.",
    "Consistency is intelligence applied daily.",
    "One disciplined day changes your trajectory.",
    "You are closer than you think.",
    "Keep the chain unbroken. The future is watching.",
    "Deep work is a promise you keep to your descendants.",
    "Small clean sessions become a life you recognize.",
    "Earn tomorrow by honoring the next 60 minutes.",
];

static EMPEROR_QUOTES: &[&str] = &[
    "You were not built for small outcomes.",
    "Discipline is sovereignty.",
    "AUGUSTINUS does not drift.",
    "Legacy is constructed in silence.",
    "Your name can outlive your biology. Act like it.",
    "The empire is built in the hours nobody applauds.",
    "You don't need permission to become inevitable.",
    "Let the work be royal. Let the routine be law.",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotivationState {
    default_tone: Tone,
    current_tone: Tone,
    rotate_every: Duration,
    rotate_elapsed: Duration,
    last_quote_index: Option<usize>,
    quote: &'static str,
    pub idle: IdleTracker,
    pub pulse: BannerPulse,
    pub typewriter: QuoteTypewriter,
    pub ticker: Ticker,
    pub particles: ParticleField,
    burst_remaining: Duration,
    cool_down_remaining: Duration,
    wake_pulse_remaining: Duration,
    ticker_width: u16,
    ticker_window: String,
}

impl MotivationState {
    pub fn new(idle_threshold: Duration) -> Self {
        let default_tone = Tone::Encouraging;
        let current_tone = default_tone;
        let quote = quote_list(current_tone)
            .first()
            .copied()
            .unwrap_or("...");
        let mut typewriter = QuoteTypewriter::new(QUOTE_TYPEWRITER_SPEED_CPS);
        typewriter.set_text(quote);
        let mut ticker = Ticker::new(TICKER_SPEED_CPS);
        ticker.set_text(DEFAULT_TICKER_TEXT);
        Self {
            default_tone,
            current_tone,
            rotate_every: DEFAULT_ROTATE_EVERY,
            rotate_elapsed: Duration::ZERO,
            last_quote_index: Some(0),
            quote,
            idle: IdleTracker::new(idle_threshold),
            pulse: BannerPulse::new(BANNER_PULSE_PERIOD),
            typewriter,
            ticker,
            particles: ParticleField::new(Seed(1), 1, 1, PARTICLE_COUNT),
            burst_remaining: Duration::ZERO,
            cool_down_remaining: Duration::ZERO,
            wake_pulse_remaining: Duration::ZERO,
            ticker_width: 0,
            ticker_window: String::new(),
        }
    }

    pub fn quote(&self) -> &'static str {
        self.quote
    }

    pub fn tone(&self) -> Tone {
        self.current_tone
    }

    pub fn on_activity(&mut self) {
        let was_idle = self.idle.is_idle();
        self.idle.on_activity();
        if was_idle {
            self.on_wake_from_idle();
            self.set_tone(self.default_tone);
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        let was_idle = self.idle.is_idle();
        self.idle.advance(dt);
        let is_idle = self.idle.is_idle();
        if !was_idle && is_idle {
            self.set_tone(Tone::Brutal);
        }

        self.rotate_elapsed = self.rotate_elapsed.saturating_add(dt);
        while self.rotate_elapsed >= self.rotate_every {
            self.rotate_elapsed -= self.rotate_every;
            self.rotate_quote();
        }

        self.pulse.tick(dt);
        self.typewriter.tick(dt);
        self.ticker.tick(dt);
        self.particles.tick(dt);

        if self.ticker_width > 0 {
            self.ticker.fill_window(self.ticker_width, &mut self.ticker_window);
        }

        self.burst_remaining = self.burst_remaining.saturating_sub(dt);
        self.cool_down_remaining = self.cool_down_remaining.saturating_sub(dt);
        self.wake_pulse_remaining = self.wake_pulse_remaining.saturating_sub(dt);
    }

    pub fn set_particle_bounds(&mut self, width: u16, height: u16) {
        self.particles.resize(width, height);
    }

    pub fn set_ticker_width(&mut self, width: u16) {
        if self.ticker_width == width {
            return;
        }
        self.ticker_width = width;
        self.ticker_window.clear();
        self.ticker_window.reserve(width as usize);
        self.ticker.fill_window(self.ticker_width, &mut self.ticker_window);
    }

    pub fn ticker_window(&self) -> &str {
        &self.ticker_window
    }

    pub fn on_focus_start(&mut self) {
        self.burst_remaining = BURST_TTL;
        self.particles.trigger_burst(24, BURST_TTL);
    }

    pub fn on_focus_stop(&mut self) {
        self.cool_down_remaining = COOL_DOWN_TTL;
    }

    pub fn on_wake_from_idle(&mut self) {
        self.wake_pulse_remaining = WAKE_PULSE_TTL;
        self.particles.trigger_burst(16, WAKE_PULSE_TTL);
    }

    pub fn burst_remaining(&self) -> Duration {
        self.burst_remaining
    }

    pub fn cool_down_remaining(&self) -> Duration {
        self.cool_down_remaining
    }

    pub fn wake_pulse_remaining(&self) -> Duration {
        self.wake_pulse_remaining
    }

    fn set_tone(&mut self, tone: Tone) {
        if self.current_tone == tone {
            return;
        }
        self.current_tone = tone;
        self.last_quote_index = None;
        self.rotate_quote();
    }

    fn rotate_quote(&mut self) {
        let list = quote_list(self.current_tone);
        if list.is_empty() {
            self.quote = "...";
            self.last_quote_index = None;
            self.typewriter.set_text(self.quote);
            return;
        }

        let mut next_index = match self.last_quote_index {
            Some(i) => (i + 1) % list.len(),
            None => 0,
        };

        if let Some(last) = self.last_quote_index {
            if list.len() > 1 && next_index == last {
                next_index = (next_index + 1) % list.len();
            }
        }

        self.quote = list[next_index];
        self.last_quote_index = Some(next_index);
        self.typewriter.set_text(self.quote);
    }
}

fn quote_list(tone: Tone) -> &'static [&'static str] {
    match tone {
        Tone::Brutal => BRUTAL_QUOTES,
        Tone::Encouraging => ENCOURAGING_QUOTES,
        Tone::Emperor => EMPEROR_QUOTES,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleTracker {
    threshold: Duration,
    elapsed: Duration,
}

impl IdleTracker {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            elapsed: Duration::ZERO,
        }
    }

    pub fn on_activity(&mut self) {
        self.elapsed = Duration::ZERO;
    }

    pub fn advance(&mut self, dt: Duration) {
        self.elapsed = self.elapsed.saturating_add(dt);
    }

    pub fn is_idle(&self) -> bool {
        self.elapsed >= self.threshold
    }
}
