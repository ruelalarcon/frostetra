use std::convert::Infallible;
use std::sync::Arc;

use bot::{BotConfig, BotOptions};
use enumset::EnumSet;
use futures::prelude::*;
use tbp::Randomizer;

use crate::bot::Bot;
use crate::data::GameState;
use crate::rules::GameRules;
use crate::sync::BotSyncronizer;
use crate::tbp::{BotMessage, FrontendMessage};

mod bot;
mod tbp;
#[macro_use]
pub mod data;
pub mod movegen;
pub mod rules;
mod search;
mod sync;

pub async fn run(
    mut incoming: impl Stream<Item = FrontendMessage> + Unpin,
    mut outgoing: impl Sink<BotMessage, Error = Infallible> + Unpin,
    config: Arc<BotConfig>,
) {
    outgoing
        .send(BotMessage::Info {
            name: "Cold Clear 2",
            version: concat!(env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH")),
            author: "MinusKelvin",
            features: &[],
        })
        .await
        .unwrap();

    let bot = Arc::new(BotSyncronizer::new());

    spawn_workers(&bot);

    let mut waiting_on_first_piece = None;
    let mut game_rules = GameRules::default();

    while let Some(msg) = incoming.next().await {
        match msg {
            FrontendMessage::Start(start) => {
                if start.hold.is_none() && start.queue.is_empty() {
                    waiting_on_first_piece = Some(start);
                } else {
                    bot.start(create_bot(start, game_rules, config.clone()));
                }
            }
            FrontendMessage::Stop => {
                bot.stop();
                waiting_on_first_piece = None;
            }
            FrontendMessage::Suggest => {
                if let Some((moves, move_info)) = bot.suggest() {
                    outgoing
                        .send(BotMessage::Suggestion { moves, move_info })
                        .await
                        .unwrap();
                }
            }
            FrontendMessage::Play { mv } => {
                bot.advance(mv);
                puffin::GlobalProfiler::lock().new_frame();
            }
            FrontendMessage::NewPiece { piece } => {
                if let Some(mut start) = waiting_on_first_piece.take() {
                    if let Randomizer::SevenBag { bag_state } = &mut start.randomizer {
                        if bag_state.is_empty() {
                            *bag_state = EnumSet::all();
                        }
                        bag_state.remove(piece);
                    }
                    start.queue.push(piece);
                    bot.start(create_bot(start, game_rules, config.clone()));
                } else {
                    bot.new_piece(piece);
                }
            }
            FrontendMessage::Rules {
                kickset,
                rot180,
                sonic_drop,
            } => {
                game_rules = GameRules {
                    kickset,
                    rot180,
                    sonic_drop,
                };
                outgoing.send(BotMessage::Ready).await.unwrap();
            }
            FrontendMessage::Quit => break,
            FrontendMessage::Unknown => {}
        }
    }
}

fn create_bot(mut start: tbp::Start, rules: GameRules, config: Arc<BotConfig>) -> Bot {
    let reserve = start.hold.unwrap_or_else(|| start.queue.remove(0));

    let speculate = matches!(start.randomizer, Randomizer::SevenBag { .. });
    let bag = match start.randomizer {
        Randomizer::Unknown => EnumSet::all(),
        Randomizer::SevenBag { mut bag_state } => {
            for &p in start.queue.iter().rev() {
                if bag_state == EnumSet::all() {
                    bag_state = EnumSet::empty();
                }
                bag_state.insert(p);
            }
            bag_state
        }
    };

    let state = GameState {
        reserve,
        back_to_back: start.back_to_back,
        combo: start.combo.try_into().unwrap_or(255),
        bag,
        board: start.board.into(),
    };

    Bot::new(
        BotOptions {
            speculate,
            rules,
            config,
        },
        state,
        &start.queue,
    )
}

fn spawn_workers(bot: &Arc<BotSyncronizer>) {
    for _ in 0..1 {
        let bot = bot.clone();
        std::thread::spawn(move || bot.work_loop());
    }
}
