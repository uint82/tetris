use std::time::Duration;

pub const MODE_COUNT: usize = 4;
pub const SPRINT_LINES: u32 = 40;
pub const TIME_LIMIT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Marathon,
    Sprint,
    Time,
    Classic,
}

impl Mode {
    pub const ALL: [Self; MODE_COUNT] = [Self::Marathon, Self::Sprint, Self::Time, Self::Classic];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Marathon => "MARATHON",
            Self::Sprint => "SPRINT",
            Self::Time => "TIME",
            Self::Classic => "CLASSIC",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Marathon => "Endless play until you top out",
            Self::Sprint => "Clear 40 lines as fast as possible",
            Self::Time => "Highest score in two minutes",
            Self::Classic => "NES-style rules and scoring",
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_timed(self) -> bool {
        matches!(self, Self::Sprint | Self::Time)
    }

    pub const fn is_classic(self) -> bool {
        matches!(self, Self::Classic)
    }

    pub const fn allows_hold(self) -> bool {
        !self.is_classic()
    }

    pub const fn shows_ghost(self) -> bool {
        !self.is_classic()
    }

    pub const fn allows_hard_drop(self) -> bool {
        !self.is_classic()
    }

    pub const fn uses_wall_kicks(self) -> bool {
        !self.is_classic()
    }

    pub const fn uses_lock_delay(self) -> bool {
        !self.is_classic()
    }

    pub const fn scores_t_spins(self) -> bool {
        !self.is_classic()
    }

    pub const fn shows_combo(self) -> bool {
        !self.is_classic()
    }

    pub const fn line_goal(self) -> Option<u32> {
        match self {
            Self::Sprint => Some(SPRINT_LINES),
            Self::Marathon | Self::Time | Self::Classic => None,
        }
    }

    pub const fn time_limit(self) -> Option<Duration> {
        match self {
            Self::Time => Some(TIME_LIMIT),
            Self::Marathon | Self::Sprint | Self::Classic => None,
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
        assert!(Mode::Time.is_timed());
        assert!(!Mode::Marathon.is_timed());
        assert!(!Mode::Classic.is_timed());
    }

    #[test]
    fn sprint_has_a_line_goal() {
        assert_eq!(Mode::Sprint.line_goal(), Some(SPRINT_LINES));
        assert_eq!(Mode::Sprint.time_limit(), None);
    }

    #[test]
    fn time_has_a_time_limit() {
        assert_eq!(Mode::Time.time_limit(), Some(TIME_LIMIT));
        assert_eq!(Mode::Time.line_goal(), None);
    }

    #[test]
    fn marathon_and_classic_have_no_objective() {
        for mode in [Mode::Marathon, Mode::Classic] {
            assert_eq!(mode.line_goal(), None);
            assert_eq!(mode.time_limit(), None);
            assert!(!mode.objective_met(SPRINT_LINES, TIME_LIMIT));
        }
    }

    #[test]
    fn classic_disables_modern_assists() {
        assert!(Mode::Classic.is_classic());
        assert!(!Mode::Classic.allows_hold());
        assert!(!Mode::Classic.shows_ghost());
        assert!(!Mode::Classic.allows_hard_drop());
        assert!(!Mode::Classic.uses_wall_kicks());
        assert!(!Mode::Classic.uses_lock_delay());
        assert!(!Mode::Classic.scores_t_spins());
        assert!(!Mode::Classic.shows_combo());
        for mode in [Mode::Marathon, Mode::Sprint, Mode::Time] {
            assert!(mode.allows_hold());
            assert!(mode.shows_ghost());
            assert!(mode.allows_hard_drop());
            assert!(mode.uses_wall_kicks());
            assert!(mode.uses_lock_delay());
            assert!(mode.scores_t_spins());
            assert!(mode.shows_combo());
        }
    }

    #[test]
    fn sprint_completes_at_the_line_goal() {
        assert!(!Mode::Sprint.objective_met(SPRINT_LINES - 1, Duration::ZERO));
        assert!(Mode::Sprint.objective_met(SPRINT_LINES, Duration::ZERO));
    }

    #[test]
    fn time_completes_at_the_time_limit() {
        assert!(!Mode::Time.objective_met(0, TIME_LIMIT - Duration::from_secs(1)));
        assert!(Mode::Time.objective_met(0, TIME_LIMIT));
    }

    #[test]
    fn remaining_time_counts_down_to_zero() {
        assert_eq!(
            Mode::Time.remaining_time(Duration::from_secs(30)),
            Some(TIME_LIMIT - Duration::from_secs(30))
        );
        assert_eq!(
            Mode::Time.remaining_time(TIME_LIMIT + Duration::from_secs(1)),
            Some(Duration::ZERO)
        );
        assert_eq!(Mode::Sprint.remaining_time(Duration::ZERO), None);
    }
}
