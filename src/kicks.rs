use crate::piece::{PieceKind, Rotation};

const O_OFFSETS: [(i32, i32); 1] = [(0, 0)];

const JLSTZ_CLOCKWISE: [[(i32, i32); 5]; 4] = [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
];

const JLSTZ_COUNTER_CLOCKWISE: [[(i32, i32); 5]; 4] = [
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
];

const I_CLOCKWISE: [[(i32, i32); 5]; 4] = [
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
];

const I_COUNTER_CLOCKWISE: [[(i32, i32); 5]; 4] = [
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
];

pub fn offsets(kind: PieceKind, from: Rotation, clockwise: bool) -> &'static [(i32, i32)] {
    match (kind, clockwise) {
        (PieceKind::O, _) => &O_OFFSETS,
        (PieceKind::I, true) => &I_CLOCKWISE[from.index()],
        (PieceKind::I, false) => &I_COUNTER_CLOCKWISE[from.index()],
        (_, true) => &JLSTZ_CLOCKWISE[from.index()],
        (_, false) => &JLSTZ_COUNTER_CLOCKWISE[from.index()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_piece_has_no_kick_offsets() {
        assert_eq!(offsets(PieceKind::O, Rotation::Spawn, true), [(0, 0)]);
    }

    #[test]
    fn i_piece_uses_its_own_table() {
        let kicks = offsets(PieceKind::I, Rotation::Spawn, true);
        assert_eq!(kicks.len(), 5);
        assert_eq!(kicks[1], (-2, 0));
    }

    #[test]
    fn jlstz_table_varies_by_direction() {
        let clockwise = offsets(PieceKind::T, Rotation::Spawn, true);
        let counter_clockwise = offsets(PieceKind::T, Rotation::Spawn, false);
        assert_ne!(clockwise, counter_clockwise);
        assert_eq!(clockwise[0], (0, 0));
    }
}
