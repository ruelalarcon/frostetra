pub mod freestyle;

use enum_dispatch::enum_dispatch;

use crate::bot::behavior::freestyle::Freestyle;
use crate::bot::{BotOptions, Statistics};
use crate::tetris::model::{Piece, Placement};

#[enum_dispatch]
pub(super) enum BehaviorEnum {
    Freestyle,
}

#[enum_dispatch(BehaviorEnum)]
pub(super) trait Behavior {
    fn advance(&mut self, options: &BotOptions, mv: Placement) -> Option<BehaviorSwitch>;
    fn new_piece(&mut self, options: &BotOptions, piece: Piece);
    fn suggest(&self, options: &BotOptions) -> Vec<Placement>;
    fn do_work(&self, options: &BotOptions) -> Statistics;
}

#[allow(dead_code)]
pub(super) enum BehaviorSwitch {
    Freestyle,
}
