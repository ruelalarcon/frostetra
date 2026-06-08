use std::collections::VecDeque;
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use enumset::EnumSet;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::data::{GameState, Piece, Placement};
use crate::rules::GameRules;

mod freestyle;

use self::freestyle::Freestyle;

pub struct Bot {
    options: BotOptions,
    current: GameState,
    queue: VecDeque<Piece>,
    mode: ModeEnum,
    bag_tracker: Option<SevenBagTracker>,
    consumed_pieces: usize,
    logged_speculation_start: bool,
    logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    pub freestyle_weights: freestyle::Weights,
    pub freestyle_exploitation: f64,
}

impl Default for BotConfig {
    fn default() -> Self {
        static DEFAULT: Lazy<BotConfig> =
            Lazy::new(|| serde_json::from_str(include_str!("default.json")).unwrap());
        DEFAULT.clone()
    }
}

#[derive(Debug)]
pub struct BotOptions {
    pub speculate: bool,
    pub rules: GameRules,
    pub config: Arc<BotConfig>,
}

#[enum_dispatch]
enum ModeEnum {
    Freestyle,
}

#[enum_dispatch(ModeEnum)]
trait Mode {
    fn advance(&mut self, options: &BotOptions, mv: Placement) -> Option<ModeSwitch>;
    fn new_piece(&mut self, options: &BotOptions, piece: Piece);
    fn suggest(&self, options: &BotOptions) -> Vec<Placement>;
    fn do_work(&self, options: &BotOptions) -> Statistics;
}

#[allow(dead_code)]
enum ModeSwitch {
    Freestyle,
}

impl Bot {
    pub fn new(
        mut options: BotOptions,
        mut root: GameState,
        queue: &[Piece],
        bag_tracker: Option<SevenBagTracker>,
    ) -> Self {
        let mut logged_speculation_start = false;
        let mut logs = Vec::new();
        if let Some(tracker) = &bag_tracker {
            if let Some(bag) = tracker.confident_bag_after(1) {
                root.bag = bag;
                options.speculate = true;
                logs.push(speculation_log(bag));
                logged_speculation_start = true;
            }
        }

        Bot {
            current: root,
            queue: queue.iter().copied().collect(),
            mode: Freestyle::new(&options, root, queue).into(),
            options,
            bag_tracker,
            consumed_pieces: 1,
            logged_speculation_start,
            logs,
        }
    }

    pub fn advance(&mut self, mv: Placement) {
        puffin::profile_function!();
        self.current
            .advance(self.queue.pop_front().unwrap(), mv, &self.options.rules);
        self.consumed_pieces += 1;
        if let Some(to) = self.mode.advance(&self.options, mv) {
            self.switch(to);
        };
        self.maybe_start_speculation();
    }

    pub fn new_piece(&mut self, piece: Piece) {
        puffin::profile_function!();
        self.queue.push_back(piece);
        if let Some(tracker) = &mut self.bag_tracker {
            tracker.observe(piece);
        }
        if !self.maybe_start_speculation() {
            self.mode.new_piece(&self.options, piece);
        }
    }

    pub fn suggest(&self) -> Vec<Placement> {
        puffin::profile_function!();
        self.mode.suggest(&self.options)
    }

    pub fn do_work(&self) -> Statistics {
        puffin::profile_function!();
        self.mode.do_work(&self.options)
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }

    fn switch(&mut self, to: ModeSwitch) {
        puffin::profile_function!();
        match to {
            ModeSwitch::Freestyle => {
                self.mode =
                    Freestyle::new(&self.options, self.current, self.queue.make_contiguous()).into()
            }
        }
    }

    fn maybe_start_speculation(&mut self) -> bool {
        if self.options.speculate {
            return false;
        }

        let Some(tracker) = &self.bag_tracker else {
            return false;
        };
        let Some(bag) = tracker.confident_bag_after(self.consumed_pieces) else {
            return false;
        };

        self.current.bag = bag;
        self.options.speculate = true;
        if !self.logged_speculation_start {
            self.logs.push(speculation_log(bag));
            self.logged_speculation_start = true;
        }
        self.mode =
            Freestyle::new(&self.options, self.current, self.queue.make_contiguous()).into();
        true
    }
}

fn speculation_log(bag: EnumSet<Piece>) -> String {
    format!(
        "seven-bag tracker confident; starting speculation with bag {:?}",
        bag
    )
}

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

#[derive(Copy, Clone, Debug)]
pub struct Statistics {
    pub nodes: u64,
    pub selections: u64,
    pub expansions: u64,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            nodes: 0,
            selections: 0,
            expansions: 0,
        }
    }
}

impl Statistics {
    pub fn accumulate(&mut self, other: Self) {
        self.nodes += other.nodes;
        self.selections += other.selections;
        self.expansions += other.expansions;
    }
}
