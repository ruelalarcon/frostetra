use std::hint::black_box;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use enumset::EnumSet;
use frostetra::bot::{Bot, BotOptions, BotRunner, Statistics};
use frostetra::config::{BotConfig, SearchBudgetConfig};
use frostetra::search::SearchBudget;
use frostetra::tetris::model::rules::GameRules;
use frostetra::tetris::model::{Board, GameState, Piece};

const QUEUE: [Piece; 5] = [Piece::T, Piece::I, Piece::O, Piece::S, Piece::L];

struct ProfileArgs {
    batches: u64,
    nodes_per_batch: u64,
    board: ProfileBoard,
    locked: bool,
    threads: NonZeroUsize,
}

#[derive(Clone, Copy)]
enum ProfileBoard {
    Empty,
    Tspin,
    Terrible,
}

pub fn run(args: impl Iterator<Item = String>) {
    let args = ProfileArgs::parse(args);
    let mut runner = runner(&args);
    let mut stats = Statistics::default();
    let started = Instant::now();

    for _ in 0..args.batches {
        stats.accumulate(black_box(
            runner.run_for(SearchBudget::Nodes(args.nodes_per_batch)),
        ));
    }

    let elapsed = started.elapsed();
    println!("batches: {}", args.batches);
    println!("nodes_per_batch: {}", args.nodes_per_batch);
    println!("nodes: {}", stats.nodes);
    println!("selections: {}", stats.selections);
    println!("expansions: {}", stats.expansions);
    println!("max_depth: {}", stats.max_depth);
    println!("elapsed_ms: {:.3}", elapsed.as_secs_f64() * 1000.0);
    println!(
        "nodes_per_second: {:.0}",
        stats.nodes as f64 / elapsed.as_secs_f64()
    );

    black_box(&mut runner);
}

impl ProfileArgs {
    fn parse(args: impl Iterator<Item = String>) -> Self {
        let mut args = args;
        let mut parsed = ProfileArgs {
            batches: 10_000,
            nodes_per_batch: 32,
            board: ProfileBoard::Tspin,
            locked: false,
            threads: NonZeroUsize::new(1).unwrap(),
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--batches" => parsed.batches = parse_next(&mut args, "--batches"),
                "--nodes" => parsed.nodes_per_batch = parse_next(&mut args, "--nodes"),
                "--board" => parsed.board = parse_board(&next_value(&mut args, "--board")),
                "--locked" => parsed.locked = true,
                "--threads" => {
                    parsed.threads = NonZeroUsize::new(parse_next(&mut args, "--threads"))
                        .expect("--threads must be greater than zero")
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => panic!("unknown search profile argument: {other}"),
            }
        }

        parsed
    }
}

fn parse_next<T: std::str::FromStr>(args: &mut impl Iterator<Item = String>, name: &str) -> T {
    next_value(args, name)
        .parse()
        .unwrap_or_else(|_| panic!("invalid value for {name}"))
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> String {
    args.next()
        .unwrap_or_else(|| panic!("missing value for {name}"))
}

fn parse_board(value: &str) -> ProfileBoard {
    match value {
        "empty" => ProfileBoard::Empty,
        "tspin" => ProfileBoard::Tspin,
        "terrible" => ProfileBoard::Terrible,
        _ => panic!("--board must be one of: empty, tspin, terrible"),
    }
}

fn print_help() {
    println!(
        "Usage: profile search [--batches N] [--nodes N] [--board empty|tspin|terrible] [--locked] [--threads N]"
    );
}

fn runner(args: &ProfileArgs) -> BotRunner<Board> {
    let bot = Bot::new(options(args), root(board(args.board)), &QUEUE, None);
    BotRunner::from_seed(bot, 0, false)
}

fn options(args: &ProfileArgs) -> BotOptions {
    let mut config = BotConfig::default();
    config.search.rng = frostetra::config::SearchRngConfig::Seeded { seed: 0 };
    config.search.threads = args.threads;
    if !args.locked {
        config.search.budget = SearchBudgetConfig::NodesPerSuggest {
            nodes: args.nodes_per_batch,
        };
    }

    BotOptions {
        speculate: false,
        rules: GameRules::default(),
        config: Arc::new(config),
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

fn board(kind: ProfileBoard) -> Board {
    match kind {
        ProfileBoard::Empty => Board::default(),
        ProfileBoard::Tspin => board_tspin(),
        ProfileBoard::Terrible => board_terrible(),
    }
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
