use enumset::EnumSet;

use crate::tetris::model::Piece;

#[derive(Clone, Debug)]
pub struct SevenBagTracker {
    candidates: Vec<Vec<EnumSet<Piece>>>,
    current_observed_index: usize,
    next_generated_index: Option<usize>,
}

impl SevenBagTracker {
    pub fn from_observed(queue: &[Piece]) -> Self {
        Self::from_observed_at_current(queue, 0)
    }

    pub fn from_observed_at_current(queue: &[Piece], current_observed_index: usize) -> Self {
        let mut tracker = SevenBagTracker {
            candidates: Vec::new(),
            current_observed_index,
            next_generated_index: None,
        };
        for &piece in queue {
            tracker.observe(piece);
        }
        tracker
    }

    pub fn from_piece_stream(
        offset: Option<usize>,
        pieces: &[Piece],
        current_observed_index: usize,
    ) -> Self {
        let mut tracker = SevenBagTracker {
            candidates: Vec::new(),
            current_observed_index,
            next_generated_index: offset,
        };
        for (i, &piece) in pieces.iter().enumerate() {
            tracker.observe_at(offset.map(|base| base + i), piece);
        }
        tracker.next_generated_index = offset.map(|base| base + pieces.len());
        tracker
    }

    pub fn observe(&mut self, piece: Piece) {
        let generated_index = self.next_generated_index;
        self.observe_at(generated_index, piece);
        if let Some(index) = &mut self.next_generated_index {
            *index += 1;
        }
    }

    fn observe_at(&mut self, generated_index: Option<usize>, piece: Piece) {
        if self.candidates.is_empty() {
            self.candidates = initial_bags(generated_index, piece)
                .into_iter()
                .filter_map(|bag| {
                    bag.contains(piece)
                        .then(|| vec![consume_from_bag(bag, piece)])
                })
                .collect();
            return;
        }

        self.candidates = self
            .candidates
            .drain(..)
            .filter_map(|mut path| {
                let bag = if generated_index.is_some_and(|index| index % 7 == 0) {
                    EnumSet::all()
                } else {
                    *path.last().expect("candidate paths are never empty")
                };
                bag.contains(piece).then(|| {
                    path.push(consume_from_bag(bag, piece));
                    path
                })
            })
            .collect();
    }

    pub fn confident_bag_after(&self, pieces_consumed: usize) -> Option<EnumSet<Piece>> {
        let index = self.current_observed_index + pieces_consumed.checked_sub(1)?;
        let mut bags = self
            .candidates
            .iter()
            .filter_map(|path| path.get(index))
            .copied();
        let first = bags.next()?;
        bags.all(|bag| bag == first).then_some(first)
    }
}

fn initial_bags(generated_index: Option<usize>, piece: Piece) -> Vec<EnumSet<Piece>> {
    if generated_index.is_some_and(|index| index % 7 == 0) {
        vec![EnumSet::all()]
    } else {
        all_initial_bags_containing(piece)
    }
}

fn all_initial_bags_containing(piece: Piece) -> Vec<EnumSet<Piece>> {
    let others: Vec<_> = (EnumSet::all() - EnumSet::only(piece)).iter().collect();
    let mut bags = Vec::with_capacity(64);
    for mask in 0..(1 << others.len()) {
        let mut bag = EnumSet::only(piece);
        for (i, other) in others.iter().copied().enumerate() {
            if mask & (1 << i) != 0 {
                bag.insert(other);
            }
        }
        bags.push(bag);
    }
    bags
}

fn consume_from_bag(mut bag: EnumSet<Piece>, piece: Piece) -> EnumSet<Piece> {
    bag.remove(piece);
    if bag.is_empty() {
        EnumSet::all()
    } else {
        bag
    }
}
