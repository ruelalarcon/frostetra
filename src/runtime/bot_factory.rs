use std::sync::Arc;

use enumset::EnumSet;

use crate::bot::{Bot, BotOptions};
use crate::config::BotConfig;
use crate::protocol::sbp::{Randomizer, Start};
use crate::tetris::model::rules::GameRules;
use crate::tetris::model::{Board, DynamicBoard, GameState, Piece};
use crate::tetris::movegen::MovegenBoard;
use crate::tetris::randomizer::seven_bag::SevenBagTracker;

pub fn create_bot(
    start: Start,
    rules: GameRules,
    randomizer: Randomizer,
    config: Arc<BotConfig>,
) -> BotInstance {
    debug_assert!(
        (4..=127).contains(&rules.board_width),
        "unsupported board width {}",
        rules.board_width
    );

    let Start {
        board,
        active,
        queue,
        hold,
        combo,
        back_to_back,
        piece_stream,
        incoming_garbage: _,
    } = start;
    let start = StartFields {
        active,
        queue,
        hold,
        combo,
        back_to_back,
        piece_stream,
    };

    if rules.board_width == 10 {
        create_typed_bot(
            start.with_board(board.into_fixed::<10>().expect("rules width checked")),
            rules,
            randomizer,
            config,
        )
        .into()
    } else {
        create_typed_bot(
            start.with_board(
                board
                    .into_dynamic_width(rules.board_width as usize)
                    .expect("rules width checked"),
            ),
            rules,
            randomizer,
            config,
        )
        .into()
    }
}

pub enum BotInstance {
    Standard(Bot<Board>),
    Dynamic(Bot<DynamicBoard>),
}

impl From<Bot<Board>> for BotInstance {
    fn from(bot: Bot<Board>) -> Self {
        BotInstance::Standard(bot)
    }
}

impl From<Bot<DynamicBoard>> for BotInstance {
    fn from(bot: Bot<DynamicBoard>) -> Self {
        BotInstance::Dynamic(bot)
    }
}

struct StartFields {
    pub active: Piece,
    pub queue: Vec<Piece>,
    pub hold: Option<Piece>,
    pub combo: u32,
    pub back_to_back: u32,
    pub piece_stream: Option<crate::protocol::sbp::PieceStream>,
}

struct StartWithBoard<B> {
    pub board: B,
    pub active: Piece,
    pub queue: Vec<Piece>,
    pub hold: Option<Piece>,
    pub combo: u32,
    pub back_to_back: u32,
    pub piece_stream: Option<crate::protocol::sbp::PieceStream>,
}

trait StartBoardExt {
    fn with_board<B>(self, board: B) -> StartWithBoard<B>;
}

impl StartBoardExt for StartFields {
    fn with_board<B>(self, board: B) -> StartWithBoard<B> {
        StartWithBoard {
            board,
            active: self.active,
            queue: self.queue,
            hold: self.hold,
            combo: self.combo,
            back_to_back: self.back_to_back,
            piece_stream: self.piece_stream,
        }
    }
}

fn create_typed_bot<B: MovegenBoard>(
    start: StartWithBoard<B>,
    rules: GameRules,
    randomizer: Randomizer,
    config: Arc<BotConfig>,
) -> Bot<B> {
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
        board: start.board,
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
