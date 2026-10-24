use crate::board::Board;
use crate::piece::{ActivePiece, PieceKind, Rotation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TSpin {
    Mini,
    Full,
}

pub fn detect(board: &Board, piece: &ActivePiece, rotation_last: bool) -> Option<TSpin> {
    if piece.kind != PieceKind::T || !rotation_last {
        return None;
    }
    let center = piece.position;
    let corners = [
        (center.x - 1, center.y - 1),
        (center.x + 1, center.y - 1),
        (center.x - 1, center.y + 1),
        (center.x + 1, center.y + 1),
    ];
    let occupied = corners.map(|(x, y)| board.blocked(x, y));
    if occupied.iter().filter(|&&corner| corner).count() < 3 {
        return None;
    }
    let [front_left, front_right] = front_corners(piece.rotation);
    let full = occupied[front_left] && occupied[front_right];
    Some(if full { TSpin::Full } else { TSpin::Mini })
}

const fn front_corners(rotation: Rotation) -> [usize; 2] {
    match rotation {
        Rotation::Spawn => [0, 1],
        Rotation::Right => [1, 3],
        Rotation::Reverse => [2, 3],
        Rotation::Left => [0, 2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::PieceKind;

    fn piece() -> ActivePiece {
        ActivePiece::new(PieceKind::T)
    }

    fn board_with_corners(corners: [(i32, i32); 3]) -> Board {
        let mut board = Board::new();
        for (x, y) in corners {
            board.set(x as usize, y as usize, PieceKind::I);
        }
        board
    }

    #[test]
    fn three_corners_with_both_front_corners_is_a_full_spin() {
        let piece = piece();
        let center = piece.position;
        let board = board_with_corners([
            (center.x - 1, center.y - 1),
            (center.x + 1, center.y - 1),
            (center.x - 1, center.y + 1),
        ]);
        assert_eq!(detect(&board, &piece, true), Some(TSpin::Full));
    }

    #[test]
    fn one_front_corner_makes_a_mini_spin() {
        let piece = piece();
        let center = piece.position;
        let board = board_with_corners([
            (center.x - 1, center.y - 1),
            (center.x - 1, center.y + 1),
            (center.x + 1, center.y + 1),
        ]);
        assert_eq!(detect(&board, &piece, true), Some(TSpin::Mini));
    }

    #[test]
    fn fewer_than_three_corners_is_not_a_spin() {
        let piece = piece();
        let center = piece.position;
        let board = board_with_corners([
            (center.x - 1, center.y - 1),
            (center.x + 1, center.y - 1),
            (0, 0),
        ]);
        assert_eq!(detect(&board, &piece, true), None);
    }

    #[test]
    fn rotation_must_be_the_last_action() {
        let piece = piece();
        let center = piece.position;
        let board = board_with_corners([
            (center.x - 1, center.y - 1),
            (center.x + 1, center.y - 1),
            (center.x - 1, center.y + 1),
        ]);
        assert_eq!(detect(&board, &piece, false), None);
    }

    #[test]
    fn other_pieces_never_register_a_t_spin() {
        let piece = ActivePiece::new(PieceKind::L);
        let board = Board::new();
        assert_eq!(detect(&board, &piece, true), None);
    }
}
