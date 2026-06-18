pub mod freestyle;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::bot::behavior::freestyle::Freestyle;
use crate::bot::{BotOptions, Statistics};
use crate::search::SearchContext;
use crate::tetris::model::{GameState, Piece, Placement};

#[enum_dispatch]
pub(super) enum BehaviorEnum {
    Freestyle,
}

impl BehaviorEnum {
    pub(super) fn new(
        kind: BehaviorKind,
        options: &BotOptions,
        root: GameState,
        queue: &[Piece],
    ) -> Self {
        match kind {
            BehaviorKind::Freestyle => Freestyle::new(options, root, queue).into(),
        }
    }

    pub(super) fn kind(&self) -> BehaviorKind {
        match self {
            BehaviorEnum::Freestyle(_) => BehaviorKind::Freestyle,
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

#[enum_dispatch(BehaviorEnum)]
pub(super) trait Behavior {
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
