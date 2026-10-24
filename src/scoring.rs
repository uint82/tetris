use crate::tspin::TSpin;

pub const LINES_PER_LEVEL: u32 = 10;

#[derive(Clone, Copy, Debug, Default)]
pub struct Clear {
    pub lines: u8,
    pub tspin: Option<TSpin>,
    pub perfect: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scoring {
    Modern,
    Classic,
}

#[derive(Clone, Copy, Debug)]
pub struct ScoreState {
    scoring: Scoring,
    score: u64,
    lines: u32,
    level: u32,
    combo: i32,
    back_to_back: bool,
}

impl ScoreState {
    pub const fn new(scoring: Scoring) -> Self {
        Self {
            scoring,
            score: 0,
            lines: 0,
            level: 1,
            combo: -1,
            back_to_back: false,
        }
    }

    pub const fn score(&self) -> u64 {
        self.score
    }

    pub const fn lines(&self) -> u32 {
        self.lines
    }

    pub const fn level(&self) -> u32 {
        self.level
    }

    pub const fn combo(&self) -> i32 {
        self.combo
    }

    pub const fn back_to_back(&self) -> bool {
        self.back_to_back
    }

    pub fn add_soft_drop(&mut self, rows: u32) {
        self.score += u64::from(rows);
    }

    pub fn add_hard_drop(&mut self, rows: u32) {
        self.score += u64::from(rows) * 2;
    }

    pub fn register(&mut self, clear: Clear) {
        match self.scoring {
            Scoring::Modern => self.register_modern(clear),
            Scoring::Classic => self.register_classic(clear),
        }
    }

    fn register_modern(&mut self, clear: Clear) {
        let level = self.level;
        let base = base_points(clear, level);
        let b2b = b2b_points(clear, self.back_to_back, level);
        let perfect = perfect_points(clear, level);
        let combo = self.advance_combo(clear.lines);
        self.score += base + b2b + perfect + combo;
        self.lines += u32::from(clear.lines);
        self.level = 1 + self.lines / LINES_PER_LEVEL;
        if clear.lines > 0 {
            self.back_to_back = is_difficult(clear);
        }
    }

    fn register_classic(&mut self, clear: Clear) {
        self.score += classic_points(clear.lines, self.level);
        self.lines += u32::from(clear.lines);
        self.level = 1 + self.lines / LINES_PER_LEVEL;
    }

    fn advance_combo(&mut self, lines: u8) -> u64 {
        if lines == 0 {
            self.combo = -1;
            return 0;
        }
        self.combo += 1;
        if self.combo <= 0 {
            return 0;
        }
        50 * self.combo as u64 * u64::from(self.level)
    }
}

impl Default for ScoreState {
    fn default() -> Self {
        Self::new(Scoring::Modern)
    }
}

fn classic_points(lines: u8, level: u32) -> u64 {
    let base = match lines {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => 0,
    };
    base * u64::from(level)
}

fn base_points(clear: Clear, level: u32) -> u64 {
    u64::from(base_value(clear)) * u64::from(level)
}

const fn base_value(clear: Clear) -> u32 {
    match (clear.tspin, clear.lines) {
        (Some(TSpin::Full), 0) => 400,
        (Some(TSpin::Full), 1) => 800,
        (Some(TSpin::Full), 2) => 1200,
        (Some(TSpin::Full), 3) => 1600,
        (Some(TSpin::Mini), 0) => 100,
        (Some(TSpin::Mini), 1) => 200,
        (Some(TSpin::Mini), 2) => 400,
        (None, 1) => 100,
        (None, 2) => 300,
        (None, 3) => 500,
        (None, 4) => 800,
        _ => 0,
    }
}

fn b2b_points(clear: Clear, active: bool, level: u32) -> u64 {
    if !active || !is_difficult(clear) {
        return 0;
    }
    base_points(clear, level) / 2
}

fn perfect_points(clear: Clear, level: u32) -> u64 {
    if !clear.perfect || clear.lines == 0 {
        return 0;
    }
    let value = match clear.lines {
        1 => 800,
        2 => 1200,
        3 => 1800,
        _ => 2000,
    };
    value * u64::from(level)
}

const fn is_difficult(clear: Clear) -> bool {
    clear.lines == 4 || (clear.tspin.is_some() && clear.lines > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear(lines: u8) -> Clear {
        Clear {
            lines,
            tspin: None,
            perfect: false,
        }
    }

    #[test]
    fn tetris_scores_guideline_value() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(clear(4));
        assert_eq!(state.score(), 800);
    }

    #[test]
    fn back_to_back_tetris_adds_half_bonus() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(clear(4));
        state.register(clear(0));
        let first = state.score();
        state.register(clear(4));
        assert_eq!(state.score() - first, 1200);
    }

    #[test]
    fn combo_multiplies_by_level() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(clear(1));
        state.register(clear(1));
        assert_eq!(state.score(), 250);
        assert_eq!(state.combo(), 1);
    }

    #[test]
    fn empty_clear_resets_combo() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(clear(1));
        state.register(clear(0));
        assert_eq!(state.combo(), -1);
    }

    #[test]
    fn full_t_spin_single_scores_more_than_a_single() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(Clear {
            lines: 1,
            tspin: Some(TSpin::Full),
            perfect: false,
        });
        assert_eq!(state.score(), 800);
    }

    #[test]
    fn mini_t_spin_double_scores_guideline_value() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(Clear {
            lines: 2,
            tspin: Some(TSpin::Mini),
            perfect: false,
        });
        assert_eq!(state.score(), 400);
    }

    #[test]
    fn perfect_clear_adds_bonus() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(Clear {
            lines: 1,
            tspin: None,
            perfect: true,
        });
        assert_eq!(state.score(), 900);
    }

    #[test]
    fn level_advances_every_ten_lines() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.register(clear(4));
        state.register(clear(4));
        state.register(clear(2));
        assert_eq!(state.lines(), 10);
        assert_eq!(state.level(), 2);
    }

    #[test]
    fn drop_bonuses_add_direct_points() {
        let mut state = ScoreState::new(Scoring::Modern);
        state.add_soft_drop(3);
        state.add_hard_drop(5);
        assert_eq!(state.score(), 13);
    }

    #[test]
    fn classic_clears_use_nes_values() {
        let mut state = ScoreState::new(Scoring::Classic);
        state.register(clear(1));
        assert_eq!(state.score(), 40);
        state.register(clear(4));
        assert_eq!(state.score(), 1240);
    }

    #[test]
    fn classic_clears_ignore_modern_bonuses() {
        let mut state = ScoreState::new(Scoring::Classic);
        state.register(Clear {
            lines: 1,
            tspin: Some(TSpin::Full),
            perfect: true,
        });
        state.register(clear(1));
        assert_eq!(state.score(), 80);
        assert_eq!(state.combo(), -1);
        assert!(!state.back_to_back());
    }

    #[test]
    fn classic_clears_scale_with_level() {
        let mut state = ScoreState::new(Scoring::Classic);
        for _ in 0..10 {
            state.register(clear(1));
        }
        assert_eq!(state.level(), 2);
        let before = state.score();
        state.register(clear(1));
        assert_eq!(state.score() - before, 80);
    }
}
