use std::time::Duration;

use crate::board::{HEIGHT, WIDTH};
use crate::piece::{ActivePiece, PieceKind};

const LINE_CLEAR_DURATION: Duration = Duration::from_millis(180);
const HARD_DROP_DURATION: Duration = Duration::from_millis(120);

#[derive(Default)]
pub struct Effects {
    line_clear: Option<LineClearFlash>,
    hard_drop: Option<HardDropTrail>,
}

impl Effects {
    pub fn line_clear(&self) -> Option<&LineClearFlash> {
        self.line_clear.as_ref()
    }

    pub fn hard_drop(&self) -> Option<&HardDropTrail> {
        self.hard_drop.as_ref()
    }

    pub fn start_line_clear(&mut self, rows: u32) {
        self.line_clear = Some(LineClearFlash {
            rows,
            elapsed: Duration::ZERO,
        });
    }

    pub fn start_hard_drop(&mut self, piece: ActivePiece, distance: i32) {
        self.hard_drop = Some(HardDropTrail::new(piece, distance));
    }

    pub fn update(&mut self, dt: Duration) {
        self.tick_line_clear(dt);
        self.tick_hard_drop(dt);
    }

    fn tick_line_clear(&mut self, dt: Duration) {
        let Some(flash) = &mut self.line_clear else {
            return;
        };
        flash.elapsed += dt;
        if flash.elapsed >= LINE_CLEAR_DURATION {
            self.line_clear = None;
        }
    }

    fn tick_hard_drop(&mut self, dt: Duration) {
        let Some(trail) = &mut self.hard_drop else {
            return;
        };
        trail.elapsed += dt;
        if trail.elapsed >= HARD_DROP_DURATION {
            self.hard_drop = None;
        }
    }
}

pub struct LineClearFlash {
    rows: u32,
    elapsed: Duration,
}

impl LineClearFlash {
    pub fn contains(&self, row: usize) -> bool {
        row < u32::BITS as usize && self.rows & (1 << row) != 0
    }

    pub fn is_bright(&self) -> bool {
        self.elapsed < LINE_CLEAR_DURATION / 2
    }
}

pub struct HardDropTrail {
    kind: PieceKind,
    cells: [[bool; WIDTH]; HEIGHT],
    elapsed: Duration,
}

impl HardDropTrail {
    fn new(piece: ActivePiece, distance: i32) -> Self {
        let mut cells = [[false; WIDTH]; HEIGHT];
        for step in 0..=distance {
            for (x, y) in piece.moved(0, step).cells() {
                if let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y))
                    && x < WIDTH
                    && y < HEIGHT
                {
                    cells[y][x] = true;
                }
            }
        }
        Self {
            kind: piece.kind,
            cells,
            elapsed: Duration::ZERO,
        }
    }

    pub fn kind(&self) -> PieceKind {
        self.kind
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        self.cells[y][x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_clear_flash_reports_rows_and_expires() {
        let mut effects = Effects::default();
        effects.start_line_clear(0b101);
        let flash = effects.line_clear().expect("flash started");
        assert!(flash.contains(0));
        assert!(!flash.contains(1));
        assert!(flash.contains(2));
        assert!(flash.is_bright());
        effects.update(Duration::from_millis(100));
        assert!(!effects.line_clear().expect("flash active").is_bright());
        effects.update(Duration::from_millis(100));
        assert!(effects.line_clear().is_none());
    }

    #[test]
    fn hard_drop_trail_marks_the_path_and_expires() {
        let mut effects = Effects::default();
        let piece = ActivePiece::new(PieceKind::O);
        effects.start_hard_drop(piece, 3);
        let trail = effects.hard_drop().expect("trail started");
        assert_eq!(trail.kind(), PieceKind::O);
        assert!(trail.contains(4, 2));
        assert!(trail.contains(4, 5));
        assert!(!trail.contains(8, 5));
        effects.update(Duration::from_millis(100));
        assert!(effects.hard_drop().is_some());
        effects.update(Duration::from_millis(20));
        assert!(effects.hard_drop().is_none());
    }
}
