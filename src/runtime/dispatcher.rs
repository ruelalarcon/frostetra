use std::convert::Infallible;
use std::sync::Arc;

use futures::channel::mpsc;
use futures::prelude::*;
use serde_json::Value;

use crate::bot::BotConfig;
use crate::protocol::sbp::{BotMessage, Capabilities, FrontendMessage, Randomizer, Start};
use crate::runtime::bot_factory::create_bot;
use crate::runtime::worker_pool::BotSyncronizer;
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
                piece_stream: true,
                spawn_position: true,
            },
        })
        .await
        .unwrap();

    let mut incoming = incoming.fuse();
    let (log_sender, log_receiver) = mpsc::unbounded();
    let mut log_receiver = log_receiver.fuse();
    let bot = Arc::new(BotSyncronizer::new(log_sender));
    spawn_workers(&bot);

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
    bot: &Arc<BotSyncronizer>,
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
            allspin_b2b,
            allclear_b2b,
            spawn_x,
            spawn_y,
        } => {
            *randomizer = rules_randomizer;
            *game_rules = GameRules {
                kickset,
                rot180,
                sonic_drop,
                allspin_b2b,
                allclear_b2b,
                spawn_x,
                spawn_y,
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

fn spawn_workers(bot: &Arc<BotSyncronizer>) {
    for _ in 0..1 {
        let bot = bot.clone();
        std::thread::spawn(move || bot.work_loop());
    }
}
