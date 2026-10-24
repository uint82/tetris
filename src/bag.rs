use rand::seq::SliceRandom;

use crate::piece::PieceKind;

pub struct Bag {
    pieces: Vec<PieceKind>,
}

impl Bag {
    pub fn new() -> Self {
        Self {
            pieces: Vec::with_capacity(PieceKind::ALL.len()),
        }
    }

    pub fn next(&mut self) -> PieceKind {
        if self.pieces.is_empty() {
            self.refill();
        }
        self.pieces.pop().expect("bag is refilled")
    }

    fn refill(&mut self) {
        self.pieces.extend_from_slice(&PieceKind::ALL);
        self.pieces.shuffle(&mut rand::rng());
    }
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bag_contains_each_piece_once() {
        let mut bag = Bag::new();
        let dealt: Vec<PieceKind> = (0..PieceKind::ALL.len()).map(|_| bag.next()).collect();
        for kind in PieceKind::ALL {
            assert_eq!(dealt.iter().filter(|&&piece| piece == kind).count(), 1);
        }
    }

    #[test]
    fn consecutive_bags_keep_producing_pieces() {
        let mut bag = Bag::new();
        let dealt: Vec<PieceKind> = (0..14).map(|_| bag.next()).collect();
        for kind in PieceKind::ALL {
            assert_eq!(dealt.iter().filter(|&&piece| piece == kind).count(), 2);
        }
    }
}
