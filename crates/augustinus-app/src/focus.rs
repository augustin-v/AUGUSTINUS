use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusState {
    active_since: Option<Instant>,
    focus_seconds_today: u64,
    streak_days: u32,
}

impl FocusState {
    pub fn new_for_test() -> Self {
        Self {
            active_since: None,
            focus_seconds_today: 0,
            streak_days: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_since.is_some()
    }

    pub fn start(&mut self, now: Instant) {
        if self.active_since.is_none() {
            self.active_since = Some(now);
        }
    }

    pub fn stop(&mut self, now: Instant) -> Option<Duration> {
        let start = self.active_since.take()?;
        Some(now.saturating_duration_since(start))
    }

    pub fn focus_seconds_today(&self) -> u64 {
        self.focus_seconds_today
    }

    pub fn add_focus_seconds_today(&mut self, seconds: u64) {
        self.focus_seconds_today = self.focus_seconds_today.saturating_add(seconds);
    }

    pub fn set_focus_seconds_today(&mut self, seconds: u64) {
        self.focus_seconds_today = seconds;
    }

    pub fn streak_days(&self) -> u32 {
        self.streak_days
    }

    pub fn set_streak_days(&mut self, days: u32) {
        self.streak_days = days;
    }
}

