use crate::piece::PieceKind;

pub struct Bag {
    pieces: Vec<PieceKind>,
    rng: BagRng,
}

impl Bag {
    pub fn new() -> Self {
        Self::seeded(rand::random())
    }

    pub fn seeded(seed: u64) -> Self {
        Self {
            pieces: Vec::with_capacity(PieceKind::ALL.len()),
            rng: BagRng::new(seed),
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
        for index in (1..self.pieces.len()).rev() {
            let other = self.rng.below(index + 1);
            self.pieces.swap(index, other);
        }
    }
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

struct BagRng {
    state: u64,
}

impl BagRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn below(&mut self, upper: usize) -> usize {
        let range = upper as u64;
        let threshold = range.wrapping_neg() % range;
        loop {
            let value = self.next();
            if value >= threshold {
                return (value % range) as usize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequence(seed: u64) -> Vec<PieceKind> {
        let mut bag = Bag::seeded(seed);
        (0..PieceKind::ALL.len() * 3).map(|_| bag.next()).collect()
    }

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

    #[test]
    fn seeded_bags_repeat_their_sequence() {
        assert_eq!(sequence(42), sequence(42));
        assert_ne!(sequence(42), sequence(43));
    }
}
