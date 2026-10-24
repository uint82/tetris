#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    pub const ALL: [Self; 7] = [
        Self::I,
        Self::O,
        Self::T,
        Self::S,
        Self::Z,
        Self::J,
        Self::L,
    ];

    pub const fn box_size(self) -> i32 {
        match self {
            Self::I => 4,
            Self::O => 2,
            Self::T | Self::S | Self::Z | Self::J | Self::L => 3,
        }
    }

    pub const fn spawn_position(self) -> Position {
        let x = match self {
            Self::O => 4,
            _ => 3,
        };
        Position { x, y: 2 }
    }

    pub fn cells(self, rotation: Rotation) -> [(i32, i32); 4] {
        let mut cells = self.spawn_cells();
        for _ in 0..rotation.index() {
            cells = cells.map(|(x, y)| (self.box_size() - 1 - y, x));
        }
        cells
    }

    const fn spawn_cells(self) -> [(i32, i32); 4] {
        match self {
            Self::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            Self::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Self::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            Self::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            Self::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            Self::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            Self::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    Spawn,
    Right,
    Reverse,
    Left,
}

impl Rotation {
    pub const fn clockwise(self) -> Self {
        match self {
            Self::Spawn => Self::Right,
            Self::Right => Self::Reverse,
            Self::Reverse => Self::Left,
            Self::Left => Self::Spawn,
        }
    }

    pub const fn counter_clockwise(self) -> Self {
        match self {
            Self::Spawn => Self::Left,
            Self::Left => Self::Reverse,
            Self::Reverse => Self::Right,
            Self::Right => Self::Spawn,
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivePiece {
    pub kind: PieceKind,
    pub rotation: Rotation,
    pub position: Position,
}

impl ActivePiece {
    pub fn new(kind: PieceKind) -> Self {
        Self {
            kind,
            rotation: Rotation::Spawn,
            position: kind.spawn_position(),
        }
    }

    pub fn cells(&self) -> [(i32, i32); 4] {
        self.kind
            .cells(self.rotation)
            .map(|(x, y)| (x + self.position.x, y + self.position.y))
    }

    pub fn occupies(&self, x: i32, y: i32) -> bool {
        self.cells().contains(&(x, y))
    }

    pub fn moved(&self, dx: i32, dy: i32) -> Self {
        Self {
            position: Position {
                x: self.position.x + dx,
                y: self.position.y + dy,
            },
            ..*self
        }
    }

    pub fn rotated(&self, rotation: Rotation) -> Self {
        Self { rotation, ..*self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut cells: [(i32, i32); 4]) -> [(i32, i32); 4] {
        cells.sort_unstable();
        cells
    }

    #[test]
    fn i_piece_rotates_into_a_vertical_column() {
        assert_eq!(
            sorted(PieceKind::I.cells(Rotation::Right)),
            [(2, 0), (2, 1), (2, 2), (2, 3)]
        );
    }

    #[test]
    fn t_piece_rotates_clockwise() {
        assert_eq!(
            sorted(PieceKind::T.cells(Rotation::Right)),
            [(1, 0), (1, 1), (1, 2), (2, 1)]
        );
    }

    #[test]
    fn four_rotations_restore_the_spawn_shape() {
        for kind in PieceKind::ALL {
            let mut cells = kind.cells(Rotation::Spawn);
            for _ in 0..4 {
                cells = cells.map(|(x, y)| (kind.box_size() - 1 - y, x));
            }
            assert_eq!(sorted(cells), sorted(kind.cells(Rotation::Spawn)));
        }
    }

    #[test]
    fn o_piece_never_changes_shape() {
        let spawn = sorted(PieceKind::O.cells(Rotation::Spawn));
        for rotation in [
            Rotation::Spawn,
            Rotation::Right,
            Rotation::Reverse,
            Rotation::Left,
        ] {
            assert_eq!(sorted(PieceKind::O.cells(rotation)), spawn);
        }
    }

    #[test]
    fn rotation_cycles_reverse_each_other() {
        let rotation = Rotation::Spawn.clockwise().clockwise();
        assert_eq!(
            rotation.counter_clockwise().counter_clockwise(),
            Rotation::Spawn
        );
    }

    #[test]
    fn active_piece_offsets_cells_by_position() {
        let piece = ActivePiece::new(PieceKind::O);
        let cells = sorted(piece.cells());
        let offset = piece.position;
        let origin = sorted(PieceKind::O.cells(Rotation::Spawn));
        assert_eq!(cells, origin.map(|(x, y)| (x + offset.x, y + offset.y)));
    }

    #[test]
    fn moved_piece_shifts_both_axes() {
        let piece = ActivePiece::new(PieceKind::T);
        let moved = piece.moved(-2, 3);
        assert_eq!(moved.position.x, piece.position.x - 2);
        assert_eq!(moved.position.y, piece.position.y + 3);
    }
}
