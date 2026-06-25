use std::num::NonZeroUsize;
use std::sync::{Arc, Barrier};
use std::thread;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use enumset::EnumSet;
use frostetra::bot::{Bot, BotOptions, BotRunner};
use frostetra::config::BotConfig;
use frostetra::search::{SearchBudget, SearchContext};
use frostetra::tetris::model::rules::GameRules;
use frostetra::tetris::model::{Board, GameState, Piece};

const QUEUE: [Piece; 5] = [Piece::T, Piece::I, Piece::O, Piece::S, Piece::L];

fn options() -> BotOptions {
    BotOptions {
        speculate: false,
        rules: GameRules::default(),
        config: Arc::new(BotConfig::default()),
    }
}

fn root(board: Board) -> GameState<Board> {
    GameState {
        board,
        bag: EnumSet::all(),
        reserve: Piece::J,
        back_to_back: 0,
        combo: 0,
    }
}

fn runner(board: Board) -> BotRunner<Board> {
    let bot = Bot::new(options(), root(board), &QUEUE, None);
    BotRunner::from_seed(bot, 0, false)
}

fn runner_with_threads(board: Board, threads: NonZeroUsize) -> BotRunner<Board> {
    let mut config = (*options().config).clone();
    config.search.threads = threads;
    let opts = BotOptions {
        speculate: false,
        rules: GameRules::default(),
        config: Arc::new(config),
    };
    let bot = Bot::new(opts, root(board), &QUEUE, None);
    BotRunner::from_seed(bot, 0, false)
}

#[rustfmt::skip]
fn board_tspin() -> Board {
    //  . . . . . . . . .[]
    //  . . . . . . . . .[]
    // [][] . . . . . .[][]
    // [][][] . . . .[][][]
    // [][][] . . .[][][][]
    // [][][][] . .[][][][]
    // [][][][] . . .[][][]
    // [][][][][] .[][][][]
    Board {
        cols: [
            0b00111111, // x = 0
            0b00111111,
            0b00011111,
            0b00000111,
            0b00000001,
            0b00000000,
            0b00001101,
            0b00011111,
            0b00111111,
            0b11111111, // x = 9
        ],
        ..Board::default()
    }
}

#[rustfmt::skip]
fn board_terrible() -> Board {
    //  . .[][][][][][][][]
    //  . .[][][][][][][][]
    //  . . . . . . . . .[]
    //  . . . . . . . . .[]
    // [][][][][][][] . .[]
    // [][][][][][][] . .[]
    // [] . . . . . . . .[]
    // [] . . . . . . . .[]
    // [] . .[][][][][][][]
    // [] . .[][][][][][][]
    // [] . . . . . . . . .
    // [] . . . . . . . . .
    Board {
        cols: [
            0b000011111111, // x = 0
            0b000011000000,
            0b110011000000,
            0b110011001100,
            0b110011001100,
            0b110011001100,
            0b110011001100,
            0b110000001100,
            0b110000001100,
            0b111111111100, // x = 9
        ],
        ..Board::default()
    }
}

fn bench_search(c: &mut Criterion) {
    c.bench_function("search/32_nodes_tspin", |b| {
        b.iter_batched(
            || runner(board_tspin()),
            |runner| black_box(runner.run_for(SearchBudget::Nodes(black_box(32)))),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("replace_board/alternating_64", |b| {
        b.iter_batched(
            || runner(Board::default()),
            |mut runner| {
                for i in 0..64 {
                    let board = if i % 2 == 0 {
                        board_tspin()
                    } else {
                        board_terrible()
                    };
                    runner.replace_board(black_box(board));
                }
                black_box(runner)
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("replace_board_search/alternating_16", |b| {
        b.iter_batched(
            || runner(Board::default()),
            |mut runner| {
                for i in 0..16 {
                    let board = if i % 2 == 0 {
                        board_tspin()
                    } else {
                        board_terrible()
                    };
                    runner.replace_board(black_box(board));
                    black_box(runner.run_for(SearchBudget::Nodes(black_box(8))));
                    black_box(runner.suggest());
                }
                black_box(runner)
            },
            BatchSize::SmallInput,
        )
    });

    // Multi-threaded scaling benchmark. Spins up `threads` worker threads
    // that each call `runner.step()` on the same shared `BotRunner`. The
    // per-step work is small (32 nodes), so this stresses lock contention
    // and per-bucket serialization.
    let bench_threads: &[NonZeroUsize] = &[
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(8).unwrap(),
        NonZeroUsize::new(16).unwrap(),
    ];
    for &threads in bench_threads {
        c.bench_function(&format!("par_search/t{}_nodes", threads.get()), |b| {
            b.iter_batched(
                || Arc::new(runner_with_threads(board_tspin(), threads)),
                |runner| {
                    let barrier = Arc::new(Barrier::new(threads.get()));
                    let mut handles = Vec::with_capacity(threads.get());
                    let iters_per_thread = (1024 / threads.get() as u64).max(1);
                    for worker in 0..threads.get() {
                        let runner = runner.clone();
                        let barrier = barrier.clone();
                        handles.push(thread::spawn(move || {
                            barrier.wait();
                            let mut total = 0u64;
                            let context = SearchContext::from_seed(black_box(worker as u64));
                            for _ in 0..iters_per_thread {
                                let mut stats = 0;
                                while stats < 32 {
                                    stats += runner.step_with_context(&context).nodes;
                                }
                                total += stats;
                            }
                            total
                        }));
                    }
                    let mut total = 0u64;
                    for h in handles {
                        total += h.join().unwrap();
                    }
                    black_box(total)
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benchmark, bench_search);
criterion_main!(benchmark);
