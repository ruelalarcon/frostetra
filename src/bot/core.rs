use std::collections::VecDeque;

use enumset::EnumSet;

use crate::bot::behavior::freestyle::Freestyle;
use crate::bot::behavior::{Behavior, BehaviorEnum, BehaviorSwitch};
use crate::bot::{BotOptions, Statistics};
use crate::tetris::model::{GameState, Piece, Placement};
use crate::tetris::randomizer::seven_bag::SevenBagTracker;

pub struct Bot {
    options: BotOptions,
    current: GameState,
    queue: VecDeque<Piece>,
    behavior: BehaviorEnum,
    bag_tracker: Option<SevenBagTracker>,
    consumed_pieces: usize,
    logged_speculation_start: bool,
    logs: Vec<String>,
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
            behavior: Freestyle::new(&options, root, queue).into(),
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
        if let Some(to) = self.behavior.advance(&self.options, mv) {
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
            self.behavior.new_piece(&self.options, piece);
        }
    }

    pub fn suggest(&self) -> Vec<Placement> {
        puffin::profile_function!();
        self.behavior.suggest(&self.options)
    }

    pub fn do_work(&self) -> Statistics {
        puffin::profile_function!();
        self.behavior.do_work(&self.options)
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }

    fn switch(&mut self, to: BehaviorSwitch) {
        puffin::profile_function!();
        match to {
            BehaviorSwitch::Freestyle => {
                self.behavior =
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
        self.behavior =
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
