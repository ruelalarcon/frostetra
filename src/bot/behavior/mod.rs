pub mod freestyle;

use serde::{Deserialize, Serialize};

use crate::bot::behavior::freestyle::Freestyle;
use crate::bot::{BotOptions, Statistics};
use crate::search::SearchContext;
use crate::tetris::model::{BoardRepresentation, GameState, Piece, Placement};
use crate::tetris::movegen::MovegenBoard;

pub(super) enum BehaviorEnum<B: BoardRepresentation> {
    Freestyle(Freestyle<B>),
}

impl<B: MovegenBoard> BehaviorEnum<B> {
    pub(super) fn new(
        kind: BehaviorKind,
        options: &BotOptions,
        root: GameState<B>,
        queue: &[Piece],
    ) -> Self {
        match kind {
            BehaviorKind::Freestyle => {
                BehaviorEnum::Freestyle(Freestyle::new(options, root, queue))
            }
        }
    }

    pub(super) fn kind(&self) -> BehaviorKind {
        match self {
            BehaviorEnum::Freestyle(_) => BehaviorKind::Freestyle,
        }
    }

    pub(super) fn advance(
        &mut self,
        options: &BotOptions,
        mv: Placement,
    ) -> Option<BehaviorSwitch> {
        match self {
            BehaviorEnum::Freestyle(behavior) => behavior.advance(options, mv),
        }
    }

    pub(super) fn new_piece(&mut self, options: &BotOptions, piece: Piece) {
        match self {
            BehaviorEnum::Freestyle(behavior) => behavior.new_piece(options, piece),
        }
    }

    pub(super) fn suggest(&self, options: &BotOptions) -> Vec<Placement> {
        match self {
            BehaviorEnum::Freestyle(behavior) => behavior.suggest(options),
        }
    }

    pub(super) fn step_search(&self, options: &BotOptions, context: &SearchContext) -> Statistics {
        match self {
            BehaviorEnum::Freestyle(behavior) => behavior.step_search(options, context),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorKind {
    Freestyle,
}

impl Default for BehaviorKind {
    fn default() -> Self {
        Self::Freestyle
    }
}

pub(super) trait Behavior<B: BoardRepresentation> {
    fn advance(&mut self, options: &BotOptions, mv: Placement) -> Option<BehaviorSwitch>;
    fn new_piece(&mut self, options: &BotOptions, piece: Piece);
    fn suggest(&self, options: &BotOptions) -> Vec<Placement>;
    fn step_search(&self, options: &BotOptions, context: &SearchContext) -> Statistics;
}

#[allow(dead_code)]
pub(super) enum BehaviorSwitch {
    To(BehaviorKind),
}

impl BehaviorSwitch {
    pub(super) fn target(&self) -> BehaviorKind {
        match self {
            BehaviorSwitch::To(kind) => *kind,
        }
    }
}
