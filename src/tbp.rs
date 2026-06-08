use serde::{Deserialize, Deserializer, Serialize};

use crate::data::{Board, Piece, Placement};
pub use crate::movegen::Kickset;
pub use crate::rules::SonicDrop;

fn default_rot180() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum FrontendMessage {
    Rules {
        #[serde(default)]
        randomizer: Randomizer,
        #[serde(default)]
        kickset: Kickset,
        #[serde(default = "default_rot180")]
        rot180: bool,
        #[serde(default)]
        sonic_drop: SonicDrop,
        #[serde(default)]
        allspin_b2b: bool,
        #[serde(default)]
        allclear_b2b: bool,
    },
    Start(Start),
    Play {
        #[serde(rename = "move")]
        mv: Placement,
    },
    NewPiece {
        piece: Piece,
    },
    Suggest,
    Stop,
    Quit,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum BotMessage {
    Info {
        name: &'static str,
        version: &'static str,
        author: &'static str,
        features: &'static [&'static str],
    },
    Ready,
    Suggestion {
        moves: Vec<Placement>,
        move_info: MoveInfo,
    },
}

#[derive(Deserialize)]
pub struct Start {
    pub board: Board,
    pub queue: Vec<Piece>,
    pub hold: Option<Piece>,
    pub combo: u32,
    #[serde(deserialize_with = "deserialize_counter")]
    pub back_to_back: u32,
    pub piece_stream: Option<PieceStream>,
}

#[derive(Deserialize)]
pub struct PieceStream {
    pub offset: Option<usize>,
    pub pieces: Vec<Piece>,
}

fn deserialize_counter<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Counter {
        Bool(bool),
        Number(u32),
    }

    Ok(match Counter::deserialize(deserializer)? {
        Counter::Bool(value) => u32::from(value),
        Counter::Number(value) => value,
    })
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Randomizer {
    SevenBag,
    #[serde(other)]
    Unknown,
}

impl Default for Randomizer {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize)]
pub struct MoveInfo {
    pub nodes: u64,
    pub nps: f64,
    pub extra: String,
}

impl From<Vec<[Option<char>; 10]>> for Board {
    fn from(v: Vec<[Option<char>; 10]>) -> Self {
        let mut cols = [0; 10];
        for x in 0..10 {
            for y in 0..40 {
                if v[y][x].is_some() {
                    cols[x] |= 1 << y;
                }
            }
        }
        Board { cols }
    }
}
