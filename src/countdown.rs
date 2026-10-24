use std::time::Duration;

pub const COUNTDOWN_TIME: Duration = Duration::from_secs(3);

pub struct Countdown {
    remaining: Duration,
}

impl Countdown {
    pub const fn new() -> Self {
        Self {
            remaining: COUNTDOWN_TIME,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.remaining = self.remaining.saturating_sub(dt);
    }

    pub fn is_active(&self) -> bool {
        self.remaining > Duration::ZERO
    }

    pub fn number(&self) -> u32 {
        let seconds = self.remaining.as_secs() as u32;
        seconds + u32::from(self.remaining.subsec_nanos() > 0)
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_in_whole_seconds() {
        let mut countdown = Countdown::new();
        assert_eq!(countdown.number(), 3);
        countdown.update(Duration::from_millis(500));
        assert_eq!(countdown.number(), 3);
        countdown.update(Duration::from_millis(500));
        assert_eq!(countdown.number(), 2);
        countdown.update(Duration::from_secs(1));
        assert_eq!(countdown.number(), 1);
        assert!(countdown.is_active());
        countdown.update(Duration::from_secs(1));
        assert!(!countdown.is_active());
    }
}
