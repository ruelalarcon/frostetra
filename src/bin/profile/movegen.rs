use std::hint::black_box;
use std::time::Instant;

use frostetra::tetris::model::rules::GameRules;
use frostetra::tetris::model::{Board, Piece};
use frostetra::tetris::movegen::find_moves;

const PIECES: [Piece; 7] = [
    Piece::I,
    Piece::J,
    Piece::L,
    Piece::O,
    Piece::S,
    Piece::T,
    Piece::Z,
];

struct ProfileArgs {
    iterations: u64,
    board: ProfileBoard,
    piece: Option<Piece>,
}

#[derive(Clone, Copy)]
enum ProfileBoard {
    Empty,
    Tspin,
    Dtd,
    Terrible,
}

pub fn run(args: impl Iterator<Item = String>) {
    let args = ProfileArgs::parse(args);
    let board = board(args.board);
    let rules = GameRules::default();
    let pieces: &[Piece] = match args.piece {
        Some(Piece::I) => &[Piece::I],
        Some(Piece::J) => &[Piece::J],
        Some(Piece::L) => &[Piece::L],
        Some(Piece::O) => &[Piece::O],
        Some(Piece::S) => &[Piece::S],
        Some(Piece::T) => &[Piece::T],
        Some(Piece::Z) => &[Piece::Z],
        None => &PIECES,
    };

    let started = Instant::now();
    let mut placements = 0usize;
    for _ in 0..args.iterations {
        for &piece in pieces {
            placements += black_box(find_moves(black_box(&board), black_box(piece), &rules)).len();
        }
    }
    let elapsed = started.elapsed();
    let calls = args.iterations * pieces.len() as u64;
    println!("iterations: {}", args.iterations);
    println!("calls: {calls}");
    println!("placements: {placements}");
    println!("elapsed_ms: {:.3}", elapsed.as_secs_f64() * 1000.0);
    println!(
        "calls_per_second: {:.0}",
        calls as f64 / elapsed.as_secs_f64()
    );
}

impl ProfileArgs {
    fn parse(args: impl Iterator<Item = String>) -> Self {
        let mut args = args;
        let mut parsed = ProfileArgs {
            iterations: 1_000_000,
            board: ProfileBoard::Terrible,
            piece: Some(Piece::T),
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--iterations" => parsed.iterations = parse_next(&mut args, "--iterations"),
                "--board" => parsed.board = parse_board(&next_value(&mut args, "--board")),
                "--piece" => parsed.piece = parse_piece(&next_value(&mut args, "--piece")),
                "--all-pieces" => parsed.piece = None,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => panic!("unknown movegen profile argument: {other}"),
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
        "dtd" => ProfileBoard::Dtd,
        "terrible" => ProfileBoard::Terrible,
        _ => panic!("--board must be one of: empty, tspin, dtd, terrible"),
    }
}

fn parse_piece(value: &str) -> Option<Piece> {
    match value {
        "I" | "i" => Some(Piece::I),
        "J" | "j" => Some(Piece::J),
        "L" | "l" => Some(Piece::L),
        "O" | "o" => Some(Piece::O),
        "S" | "s" => Some(Piece::S),
        "T" | "t" => Some(Piece::T),
        "Z" | "z" => Some(Piece::Z),
        "all" => None,
        _ => panic!("--piece must be one of: I, J, L, O, S, T, Z, all"),
    }
}

fn print_help() {
    println!(
        "Usage: profile movegen [--iterations N] [--board empty|tspin|dtd|terrible] [--piece I|J|L|O|S|T|Z|all]"
    );
}

fn board(kind: ProfileBoard) -> Board {
    match kind {
        ProfileBoard::Empty => Board::default(),
        ProfileBoard::Tspin => board_tspin(),
        ProfileBoard::Dtd => board_dtd(),
        ProfileBoard::Terrible => board_terrible(),
    }
}

#[rustfmt::skip]
fn board_tspin() -> Board {
    Board {
        //  . . . . . . . . .[]
        //  . . . . . . . . .[]
        // [][] . . . . . .[][]
        // [][][] . . . .[][][]
        // [][][] . . .[][][][]
        // [][][][] . .[][][][]
        // [][][][] . . .[][][]
        // [][][][][] .[][][][]
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
fn board_dtd() -> Board {
    Board {
        // [][] . . . . . . . .
        // [][][][] . .[][][][]
        // [][][][] . . .[][][]
        // [][][][][][] .[][][]
        // [][][][][] . .[][][]
        // [][][][][] . . .[][]
        // [][][][][][] .[][][]
        // [][][][][][] .[][][]
        // [][][][][] .[][][][]
        cols: [
            0b111111111, // x = 0
            0b111111111,
            0b011111111,
            0b011111111,
            0b000111111,
            0b000100110,
            0b010000001,
            0b011110111,
            0b011111111,
            0b011111111, // x = 9
        ],
        ..Board::default()
    }
}

#[rustfmt::skip]
fn board_terrible() -> Board {
    Board {
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
