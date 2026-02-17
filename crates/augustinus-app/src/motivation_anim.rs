use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BannerPulse {
    period: Duration,
    elapsed: Duration,
}

impl BannerPulse {
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            elapsed: Duration::ZERO,
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        self.elapsed = self.elapsed.saturating_add(dt);
    }

    pub fn intensity_0_to_255(&self) -> u8 {
        let period_nanos = self.period.as_nanos();
        if period_nanos == 0 {
            return 255;
        }

        let half_nanos = period_nanos / 2;
        if half_nanos == 0 {
            return 255;
        }

        let t = self.elapsed.as_nanos() % period_nanos;
        let up = if t <= half_nanos { t } else { period_nanos - t };
        ((up.saturating_mul(255)) / half_nanos) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteTypewriter {
    cps: u32,
    text: &'static str,
    visible: usize,
    carry: u128,
}

impl QuoteTypewriter {
    pub fn new(cps: u32) -> Self {
        Self {
            cps,
            text: "",
            visible: 0,
            carry: 0,
        }
    }

    pub fn set_text(&mut self, text: &'static str) {
        self.text = text;
        self.visible = 0;
        self.carry = 0;
    }

    pub fn tick(&mut self, dt: Duration) {
        let len = self.text.chars().count();
        if self.visible >= len || self.cps == 0 {
            return;
        }

        let nanos = dt.as_nanos();
        let add = nanos.saturating_mul(self.cps as u128).saturating_add(self.carry);
        let inc = add / 1_000_000_000;
        self.carry = add % 1_000_000_000;

        self.visible = (self.visible.saturating_add(inc as usize)).min(len);
    }

    pub fn visible_len(&self) -> usize {
        self.visible
    }

    pub fn visible_text(&self) -> &'static str {
        let len = self.visible.min(self.text.chars().count());
        match self.text.char_indices().nth(len) {
            Some((idx, _)) => &self.text[..idx],
            None => self.text,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticker {
    cps: u32,
    text: &'static str,
    offset_cols: u32,
    carry: u128,
}

impl Ticker {
    pub fn new(cps: u32) -> Self {
        Self {
            cps,
            text: "",
            offset_cols: 0,
            carry: 0,
        }
    }

    pub fn set_text(&mut self, text: &'static str) {
        self.text = text;
        self.offset_cols = 0;
        self.carry = 0;
    }

    pub fn tick(&mut self, dt: Duration) {
        if self.cps == 0 {
            return;
        }

        let nanos = dt.as_nanos();
        let add = nanos.saturating_mul(self.cps as u128).saturating_add(self.carry);
        let inc = add / 1_000_000_000;
        self.carry = add % 1_000_000_000;
        self.offset_cols = self.offset_cols.saturating_add(inc as u32);
    }

    pub fn offset_cols(&self) -> u32 {
        self.offset_cols
    }

    pub fn window(&self, width: u16) -> String {
        let width = width as usize;
        if width == 0 {
            return String::new();
        }

        if self.text.is_empty() {
            return " ".repeat(width);
        }

        let len = self.text.chars().count();
        if len == 0 {
            return " ".repeat(width);
        }

        let start = (self.offset_cols as usize) % len;
        self.text
            .chars()
            .cycle()
            .skip(start)
            .take(width)
            .collect()
    }
}
