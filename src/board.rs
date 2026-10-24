use crate::piece::{ActivePiece, PieceKind};

pub const WIDTH: usize = 10;
pub const VISIBLE_HEIGHT: usize = 20;
pub const HIDDEN_ROWS: usize = 2;
pub const HEIGHT: usize = VISIBLE_HEIGHT + HIDDEN_ROWS;

pub struct Board {
    cells: [[Option<PieceKind>; WIDTH]; HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[None; WIDTH]; HEIGHT],
        }
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<PieceKind> {
        self.cells[y][x]
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            return true;
        }
        if y < 0 {
            return false;
        }
        self.cells[y as usize][x as usize].is_some()
    }

    pub fn fits(&self, piece: &ActivePiece) -> bool {
        piece.cells().iter().all(|&(x, y)| !self.blocked(x, y))
    }

    pub fn lock(&mut self, piece: &ActivePiece) {
        for (x, y) in piece.cells() {
            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                self.cells[y as usize][x as usize] = Some(piece.kind);
            }
        }
    }

    pub fn clear_lines(&mut self) -> u8 {
        let mut cleared = 0;
        let mut write = HEIGHT;
        for read in (0..HEIGHT).rev() {
            if self.row_full(read) {
                cleared += 1;
            } else {
                write -= 1;
                self.cells[write] = self.cells[read];
            }
        }
        for row in &mut self.cells[..write] {
            *row = [None; WIDTH];
        }
        cleared
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(Option::is_none))
    }

    fn row_full(&self, y: usize) -> bool {
        self.cells[y].iter().all(Option::is_some)
    }

    #[cfg(test)]
    pub(crate) fn set(&mut self, x: usize, y: usize, kind: PieceKind) {
        self.cells[y][x] = Some(kind);
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_accepts_a_piece() {
        let board = Board::new();
        let piece = ActivePiece::new(PieceKind::T);
        assert!(board.fits(&piece));
    }

    #[test]
    fn walls_and_floor_block_movement() {
        let board = Board::new();
        assert!(board.blocked(-1, 0));
        assert!(board.blocked(WIDTH as i32, 0));
        assert!(board.blocked(0, HEIGHT as i32));
        assert!(!board.blocked(0, -1));
    }

    #[test]
    fn occupied_cells_block_movement() {
        let mut board = Board::new();
        let piece = ActivePiece::new(PieceKind::O);
        let (x, y) = piece.cells()[0];
        board.set(x as usize, y as usize, PieceKind::I);
        assert!(!board.fits(&piece));
    }

    #[test]
    fn clearing_a_row_collapses_rows_above() {
        let mut board = Board::new();
        for x in 0..WIDTH {
            board.set(x, HEIGHT - 1, PieceKind::I);
            board.set(x, HEIGHT - 2, PieceKind::O);
        }
        board.cells[HEIGHT - 2][0] = None;
        assert_eq!(board.clear_lines(), 1);
        assert_eq!(board.cell(0, HEIGHT - 1), None);
        assert_eq!(board.cell(1, HEIGHT - 1), Some(PieceKind::O));
    }

    #[test]
    fn clearing_multiple_rows_reports_count() {
        let mut board = Board::new();
        for y in [HEIGHT - 1, HEIGHT - 2] {
            for x in 0..WIDTH {
                board.set(x, y, PieceKind::T);
            }
        }
        assert_eq!(board.clear_lines(), 2);
        assert!(board.is_empty());
    }

    #[test]
    fn incomplete_rows_are_kept() {
        let mut board = Board::new();
        for x in 0..WIDTH - 1 {
            board.set(x, HEIGHT - 1, PieceKind::L);
        }
        assert_eq!(board.clear_lines(), 0);
        assert!(!board.is_empty());
    }
}
