use std::collections::VecDeque;

use enumset::EnumSet;

use crate::bot::behavior::{BehaviorEnum, BehaviorKind, BehaviorSwitch};
use crate::bot::{BotOptions, Statistics};
use crate::search::SearchContext;
use crate::tetris::model::{Board, BoardRepresentation, GameState, Piece, Placement};
use crate::tetris::movegen::MovegenBoard;
use crate::tetris::randomizer::seven_bag::SevenBagTracker;

pub struct Bot<B: BoardRepresentation = Board> {
    options: BotOptions,
    current: GameState<B>,
    queue: VecDeque<Piece>,
    behavior: BehaviorEnum<B>,
    bag_tracker: Option<SevenBagTracker>,
    consumed_pieces: usize,
    logged_speculation_start: bool,
    logs: Vec<String>,
}

impl<B: MovegenBoard> Bot<B> {
    pub fn new(
        mut options: BotOptions,
        mut root: GameState<B>,
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
            current: root.clone(),
            queue: queue.iter().copied().collect(),
            behavior: BehaviorEnum::new(options.config.behavior.initial, &options, root, queue),
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

    pub fn replace_board(&mut self, board: B) {
        puffin::profile_function!();
        self.current.board = board;
        self.rebuild_behavior(self.behavior.kind());
    }

    pub fn board_width(&self) -> usize {
        self.current.board.width()
    }

    pub fn suggest(&self) -> Vec<Placement> {
        puffin::profile_function!();
        self.behavior.suggest(&self.options)
    }

    pub fn step_search(&self, context: &SearchContext) -> Statistics {
        puffin::profile_function!();
        self.behavior.step_search(&self.options, context)
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }

    fn switch(&mut self, to: BehaviorSwitch) {
        puffin::profile_function!();
        let from = self.behavior.kind();
        let target = to.target();
        if from == target {
            return;
        }
        self.logs.push(format!(
            "behavior switched: {} -> {}",
            from.name(),
            target.name()
        ));
        self.rebuild_behavior(target);
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
        self.rebuild_behavior(self.behavior.kind());
        true
    }

    fn rebuild_behavior(&mut self, kind: BehaviorKind) {
        self.behavior = BehaviorEnum::new(
            kind,
            &self.options,
            self.current.clone(),
            self.queue.make_contiguous(),
        );
    }
}

fn speculation_log(bag: EnumSet<Piece>) -> String {
    format!(
        "seven-bag tracker confident; starting speculation with bag {:?}",
        bag
    )
}
