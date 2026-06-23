use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use enumset::EnumSet;
use frostetra::bot::{Bot, BotOptions, BotRunner};
use frostetra::config::BotConfig;
use frostetra::search::SearchBudget;
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
}

criterion_group!(benchmark, bench_search);
criterion_main!(benchmark);
