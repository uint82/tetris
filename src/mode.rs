use std::time::Duration;

pub const MODE_COUNT: usize = 3;
pub const SPRINT_LINES: u32 = 40;
pub const ULTRA_TIME: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Marathon,
    Sprint,
    Ultra,
}

impl Mode {
    pub const ALL: [Self; MODE_COUNT] = [Self::Marathon, Self::Sprint, Self::Ultra];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Marathon => "MARATHON",
            Self::Sprint => "SPRINT",
            Self::Ultra => "ULTRA",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Marathon => "Endless play until you top out",
            Self::Sprint => "Clear 40 lines as fast as possible",
            Self::Ultra => "Highest score in two minutes",
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_timed(self) -> bool {
        matches!(self, Self::Sprint | Self::Ultra)
    }

    pub const fn line_goal(self) -> Option<u32> {
        match self {
            Self::Sprint => Some(SPRINT_LINES),
            Self::Marathon | Self::Ultra => None,
        }
    }

    pub const fn time_limit(self) -> Option<Duration> {
        match self {
            Self::Ultra => Some(ULTRA_TIME),
            Self::Marathon | Self::Sprint => None,
        }
    }

    pub fn objective_met(self, lines: u32, elapsed: Duration) -> bool {
        match (self.line_goal(), self.time_limit()) {
            (Some(goal), _) => lines >= goal,
            (None, Some(limit)) => elapsed >= limit,
            (None, None) => false,
        }
    }

    pub fn remaining_time(self, elapsed: Duration) -> Option<Duration> {
        self.time_limit().map(|limit| limit.saturating_sub(elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_have_distinct_labels_and_descriptions() {
        for (index, mode) in Mode::ALL.iter().enumerate() {
            for other in &Mode::ALL[index + 1..] {
                assert_ne!(mode.label(), other.label());
                assert_ne!(mode.description(), other.description());
            }
        }
    }

    #[test]
    fn timed_modes_require_a_countdown() {
        assert!(Mode::Sprint.is_timed());
        assert!(Mode::Ultra.is_timed());
        assert!(!Mode::Marathon.is_timed());
    }

    #[test]
    fn sprint_has_a_line_goal() {
        assert_eq!(Mode::Sprint.line_goal(), Some(SPRINT_LINES));
        assert_eq!(Mode::Sprint.time_limit(), None);
    }

    #[test]
    fn ultra_has_a_time_limit() {
        assert_eq!(Mode::Ultra.time_limit(), Some(ULTRA_TIME));
        assert_eq!(Mode::Ultra.line_goal(), None);
    }

    #[test]
    fn marathon_has_no_objective() {
        assert_eq!(Mode::Marathon.line_goal(), None);
        assert_eq!(Mode::Marathon.time_limit(), None);
        assert!(!Mode::Marathon.objective_met(SPRINT_LINES, ULTRA_TIME));
    }

    #[test]
    fn sprint_completes_at_the_line_goal() {
        assert!(!Mode::Sprint.objective_met(SPRINT_LINES - 1, Duration::ZERO));
        assert!(Mode::Sprint.objective_met(SPRINT_LINES, Duration::ZERO));
    }

    #[test]
    fn ultra_completes_at_the_time_limit() {
        assert!(!Mode::Ultra.objective_met(0, ULTRA_TIME - Duration::from_secs(1)));
        assert!(Mode::Ultra.objective_met(0, ULTRA_TIME));
    }

    #[test]
    fn remaining_time_counts_down_to_zero() {
        assert_eq!(
            Mode::Ultra.remaining_time(Duration::from_secs(30)),
            Some(ULTRA_TIME - Duration::from_secs(30))
        );
        assert_eq!(
            Mode::Ultra.remaining_time(ULTRA_TIME + Duration::from_secs(1)),
            Some(Duration::ZERO)
        );
        assert_eq!(Mode::Sprint.remaining_time(Duration::ZERO), None);
    }
}
