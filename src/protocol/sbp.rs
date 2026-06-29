use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub use crate::tetris::model::rules::{BackToBackSource, SonicDrop, SpinDetection};
use crate::tetris::model::{BoardSnapshot, Piece, Placement};
pub use crate::tetris::movegen::Kickset;

fn default_rot180() -> bool {
    true
}

fn default_spawn_x() -> i8 {
    4
}

fn default_spawn_y() -> i8 {
    20
}

fn default_board_width() -> u8 {
    10
}

fn default_board_height() -> u8 {
    40
}

#[derive(Clone, Copy, Deserialize)]
pub struct SpawnPosition {
    #[serde(default = "default_spawn_x")]
    pub x: i8,
    #[serde(default = "default_spawn_y")]
    pub y: i8,
}

impl Default for SpawnPosition {
    fn default() -> Self {
        Self {
            x: default_spawn_x(),
            y: default_spawn_y(),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
pub struct BoardSize {
    #[serde(default = "default_board_width")]
    pub width: u8,
    #[serde(default = "default_board_height")]
    pub height: u8,
}

impl Default for BoardSize {
    fn default() -> Self {
        Self {
            width: default_board_width(),
            height: default_board_height(),
        }
    }
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
        spin_detection: SpinDetection,
        #[serde(default = "default_back_to_back_sources")]
        back_to_back_sources: Vec<BackToBackSource>,
        #[serde(default)]
        spawn_position: SpawnPosition,
        #[serde(default)]
        board_size: BoardSize,
    },
    Start(Start),
    Board {
        board: BoardSnapshot,
    },
    Advance {
        #[serde(rename = "move")]
        mv: Placement,
    },
    NewPiece {
        piece: Piece,
    },
    Suggest {
        #[serde(default)]
        incoming_garbage: Option<Vec<u32>>,
    },
    Stop,
    Quit,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum BotMessage {
    Register {
        name: &'static str,
        version: &'static str,
        author: &'static str,
        capabilities: Capabilities,
    },
    Info {
        topic: &'static str,
        data: Value,
    },
    Ready,
    Suggestion {
        moves: Vec<Placement>,
    },
}

#[derive(Serialize)]
pub struct Capabilities {
    pub randomizers: &'static [&'static str],
    pub kicksets: &'static [&'static str],
    pub rot180: bool,
    pub sonic_drop: &'static [&'static str],
    pub spin_detection: &'static [&'static str],
    pub back_to_back_sources: &'static [&'static str],
    pub piece_stream: bool,
    pub spawn_position: bool,
    pub board: bool,
    pub board_size: BoardSizeCapability,
}

#[derive(Serialize)]
pub struct BoardSizeCapability {
    pub width: IntRangeCapability,
    pub height: IntRangeCapability,
}

#[derive(Serialize)]
pub struct IntRangeCapability {
    pub min: u8,
    pub max: u8,
}

fn default_back_to_back_sources() -> Vec<BackToBackSource> {
    vec![
        BackToBackSource::Quad,
        BackToBackSource::TSpin,
        BackToBackSource::TSpinMini,
    ]
}

#[derive(Deserialize)]
pub struct Start {
    pub board: BoardSnapshot,
    pub active: Piece,
    pub queue: Vec<Piece>,
    pub hold: Option<Piece>,
    pub combo: u32,
    #[serde(deserialize_with = "deserialize_counter")]
    pub back_to_back: u32,
    pub piece_stream: Option<PieceStream>,
    #[serde(default)]
    pub incoming_garbage: Option<Vec<u32>>,
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
pub struct SearchInfo {
    pub nodes: u64,
    pub nps: f64,
    pub extra: String,
}
