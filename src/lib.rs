use std::convert::Infallible;
use std::sync::Arc;

use bot::{BotConfig, BotOptions, SevenBagTracker};
use enumset::EnumSet;
use futures::prelude::*;
use serde_json::Value;
use tbp::Randomizer;

use crate::bot::Bot;
use crate::data::GameState;
use crate::rules::GameRules;
use crate::sync::BotSyncronizer;
use crate::tbp::{BotMessage, Capabilities, FrontendMessage};

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
        .send(BotMessage::Register {
            name: "Cold Clear 2",
            version: concat!(env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH")),
            author: "MinusKelvin",
            capabilities: Capabilities {
                randomizers: &["seven_bag"],
                kicksets: &["srs"],
                rot180: true,
                sonic_drop: &["only", "allow"],
                piece_stream: true,
            },
        })
        .await
        .unwrap();

    let bot = Arc::new(BotSyncronizer::new());

    spawn_workers(&bot);

    let mut waiting_on_first_piece = None;
    let mut game_rules = GameRules::default();
    let mut randomizer = Randomizer::default();

    while let Some(msg) = incoming.next().await {
        match msg {
            FrontendMessage::Start(start) => {
                if start.hold.is_none() && start.queue.is_empty() {
                    waiting_on_first_piece = Some(start);
                } else {
                    bot.start(create_bot(start, game_rules, randomizer, config.clone()));
                    send_logs(&mut outgoing, bot.drain_logs()).await;
                }
            }
            FrontendMessage::Stop => {
                bot.stop();
                waiting_on_first_piece = None;
            }
            FrontendMessage::Suggest => {
                if let Some((moves, data)) = bot.suggest() {
                    outgoing
                        .send(BotMessage::Info {
                            topic: "search",
                            data: serde_json::to_value(data).unwrap(),
                        })
                        .await
                        .unwrap();
                    outgoing
                        .send(BotMessage::Suggestion { moves })
                        .await
                        .unwrap();
                }
            }
            FrontendMessage::Play { mv } => {
                bot.advance(mv);
                send_logs(&mut outgoing, bot.drain_logs()).await;
                puffin::GlobalProfiler::lock().new_frame();
            }
            FrontendMessage::NewPiece { piece } => {
                if let Some(mut start) = waiting_on_first_piece.take() {
                    start.queue.push(piece);
                    bot.start(create_bot(start, game_rules, randomizer, config.clone()));
                    send_logs(&mut outgoing, bot.drain_logs()).await;
                } else {
                    bot.new_piece(piece);
                    send_logs(&mut outgoing, bot.drain_logs()).await;
                }
            }
            FrontendMessage::Rules {
                randomizer: rules_randomizer,
                kickset,
                rot180,
                sonic_drop,
                allspin_b2b,
                allclear_b2b,
            } => {
                randomizer = rules_randomizer;
                game_rules = GameRules {
                    kickset,
                    rot180,
                    sonic_drop,
                    allspin_b2b,
                    allclear_b2b,
                };
                outgoing.send(BotMessage::Ready).await.unwrap();
            }
            FrontendMessage::Quit => break,
            FrontendMessage::Unknown => {}
        }
    }
}

async fn send_logs(
    outgoing: &mut (impl Sink<BotMessage, Error = Infallible> + Unpin),
    logs: Vec<String>,
) {
    for data in logs {
        outgoing
            .send(BotMessage::Info {
                topic: "log",
                data: Value::String(data),
            })
            .await
            .unwrap();
    }
}

fn create_bot(
    start: tbp::Start,
    rules: GameRules,
    randomizer: Randomizer,
    config: Arc<BotConfig>,
) -> Bot {
    let visible_queue_len = start.queue.len() + 1;
    let reserve = start.hold.unwrap_or(start.active.piece);
    let mut bot_queue = Vec::with_capacity(start.queue.len() + usize::from(start.hold.is_some()));
    if start.hold.is_some() {
        bot_queue.push(start.active.piece);
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
                observed.push(start.active.piece);
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

fn spawn_workers(bot: &Arc<BotSyncronizer>) {
    for _ in 0..1 {
        let bot = bot.clone();
        std::thread::spawn(move || bot.work_loop());
    }
}
