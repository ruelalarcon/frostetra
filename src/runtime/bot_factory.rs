use std::sync::Arc;

use enumset::EnumSet;

use crate::bot::{Bot, BotOptions};
use crate::config::BotConfig;
use crate::protocol::sbp::{Randomizer, Start};
use crate::tetris::model::rules::GameRules;
use crate::tetris::model::GameState;
use crate::tetris::randomizer::seven_bag::SevenBagTracker;

pub fn create_bot(
    start: Start,
    rules: GameRules,
    randomizer: Randomizer,
    config: Arc<BotConfig>,
) -> Bot {
    let visible_queue_len = start.queue.len() + 1;
    let reserve = start.hold.unwrap_or(start.active);
    let mut bot_queue = Vec::with_capacity(start.queue.len() + usize::from(start.hold.is_some()));
    if start.hold.is_some() {
        bot_queue.push(start.active);
    }
    bot_queue.extend_from_slice(&start.queue);

    let bag_tracker = match randomizer {
        Randomizer::SevenBag => {
            if let Some(piece_stream) = start
                .piece_stream
                .as_ref()
                .filter(|stream| !stream.pieces.is_empty())
            {
                let current_observed_index =
                    piece_stream.pieces.len().saturating_sub(visible_queue_len);
                Some(SevenBagTracker::from_piece_stream(
                    piece_stream.offset,
                    &piece_stream.pieces,
                    current_observed_index,
                ))
            } else {
                let mut observed = Vec::with_capacity(start.queue.len() + 1);
                observed.push(start.active);
                observed.extend_from_slice(&start.queue);
                Some(SevenBagTracker::from_observed(&observed))
            }
        }
        Randomizer::Unknown => None,
    };

    let state = GameState {
        reserve,
        back_to_back: start.back_to_back.try_into().unwrap_or(255),
        combo: start.combo.try_into().unwrap_or(255),
        bag: EnumSet::all(),
        board: start.board.into(),
    };

    Bot::new(
        BotOptions {
            speculate: false,
            rules,
            config,
        },
        state,
        &bot_queue,
        bag_tracker,
    )
}
