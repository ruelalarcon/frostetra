use std::convert::Infallible;
use std::sync::Arc;

use enumset::EnumSet;
use futures::channel::mpsc;
use futures::prelude::*;
use serde_json::Value;

use crate::config::BotConfig;
use crate::protocol::sbp::{
    BoardSizeCapability, BotMessage, Capabilities, FrontendMessage, IntRangeCapability, Randomizer,
    Start,
};
use crate::runtime::bot_factory::create_bot;
use crate::runtime::bot_session::BotSession;
use crate::tetris::model::rules::GameRules;

pub async fn run(
    incoming: impl Stream<Item = FrontendMessage> + Unpin,
    mut outgoing: impl Sink<BotMessage, Error = Infallible> + Unpin,
    config: Arc<BotConfig>,
) {
    outgoing
        .send(BotMessage::Register {
            name: "Frostetra",
            version: concat!(env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH")),
            author: "Ruel Nathaniel Alarcon",
            capabilities: Capabilities {
                randomizers: &["seven_bag"],
                kicksets: &["srs", "srs_plus"],
                rot180: true,
                sonic_drop: &["only", "allow"],
                spin_detection: &[
                    "none",
                    "t-spins",
                    "t-spins+",
                    "all",
                    "all+",
                    "all-mini",
                    "all-mini+",
                    "mini-only",
                ],
                back_to_back_sources: &[
                    "quad",
                    "t-spin",
                    "t-spin-mini",
                    "allspin",
                    "allspin-mini",
                    "perfect-clear",
                ],
                piece_stream: true,
                spawn_position: true,
                board: true,
                board_size: BoardSizeCapability {
                    width: IntRangeCapability { min: 4, max: 127 },
                    height: IntRangeCapability { min: 1, max: 64 },
                },
            },
        })
        .await
        .unwrap();

    let mut incoming = incoming.fuse();
    let (log_sender, log_receiver) = mpsc::unbounded();
    let mut log_receiver = log_receiver.fuse();
    let bot = Arc::new(BotSession::new(log_sender, config.search.clone()));
    if bot.starts_worker() {
        spawn_workers(&bot, config.search.threads);
    }

    let mut waiting_on_first_piece = None;
    let mut game_rules = GameRules::default();
    let mut randomizer = Randomizer::default();

    loop {
        futures::select! {
            msg = incoming.next() => {
                let Some(msg) = msg else {
                    break;
                };
                if !handle_frontend_message(
                    msg,
                    &mut outgoing,
                    &bot,
                    &mut waiting_on_first_piece,
                    &mut game_rules,
                    &mut randomizer,
                    config.clone(),
                ).await {
                    break;
                }
            }
            log = log_receiver.next() => {
                let Some(log) = log else {
                    continue;
                };
                send_logs(&mut outgoing, vec![log]).await;
            }
        }
    }
}

async fn handle_frontend_message(
    msg: FrontendMessage,
    outgoing: &mut (impl Sink<BotMessage, Error = Infallible> + Unpin),
    bot: &Arc<BotSession>,
    waiting_on_first_piece: &mut Option<Start>,
    game_rules: &mut GameRules,
    randomizer: &mut Randomizer,
    config: Arc<BotConfig>,
) -> bool {
    match msg {
        FrontendMessage::Start(start) => {
            if start.hold.is_none() && start.queue.is_empty() {
                *waiting_on_first_piece = Some(start);
            } else {
                bot.start(create_bot(start, *game_rules, *randomizer, config.clone()));
                send_logs(outgoing, bot.drain_logs()).await;
            }
        }
        FrontendMessage::Stop => {
            bot.stop();
            *waiting_on_first_piece = None;
        }
        FrontendMessage::Board { board } => {
            bot.replace_board(board);
            send_logs(outgoing, bot.drain_logs()).await;
        }
        FrontendMessage::Suggest { .. } => {
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
            send_logs(outgoing, bot.drain_logs()).await;
            puffin::GlobalProfiler::lock().new_frame();
        }
        FrontendMessage::NewPiece { piece } => {
            if let Some(mut start) = waiting_on_first_piece.take() {
                start.queue.push(piece);
                bot.start(create_bot(start, *game_rules, *randomizer, config.clone()));
                send_logs(outgoing, bot.drain_logs()).await;
            } else {
                bot.new_piece(piece);
                send_logs(outgoing, bot.drain_logs()).await;
            }
        }
        FrontendMessage::Rules {
            randomizer: rules_randomizer,
            kickset,
            rot180,
            sonic_drop,
            spin_detection,
            back_to_back_sources,
            spawn_position,
            board_size,
        } => {
            *randomizer = rules_randomizer;
            *game_rules = GameRules {
                kickset,
                rot180,
                sonic_drop,
                spin_detection,
                back_to_back_sources: back_to_back_sources.into_iter().collect::<EnumSet<_>>(),
                spawn_x: spawn_position.x,
                spawn_y: spawn_position.y,
                board_width: board_size.width,
                board_height: board_size.height,
            };
            outgoing.send(BotMessage::Ready).await.unwrap();
        }
        FrontendMessage::Quit => return false,
        FrontendMessage::Unknown => {}
    }
    true
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

fn spawn_workers(bot: &Arc<BotSession>, threads: std::num::NonZeroUsize) {
    for worker in 0..threads.get() {
        let bot = bot.clone();
        std::thread::spawn(move || bot.work_loop(worker));
    }
}
