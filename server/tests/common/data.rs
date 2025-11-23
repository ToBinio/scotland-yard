use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transport {
    pub taxi: u32,
    pub bus: u32,
    pub underground: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerGame {
    pub color: String,
    pub station_id: u32,
    pub available_transport: Transport,
}

#[derive(Debug, Deserialize)]
pub struct Abilities {
    pub double_move: u32,
    pub hidden: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Move {
    Taxi,
    Bus,
    Underground,
    Hidden,
}

#[derive(Debug, Deserialize)]
pub struct MisterXGame {
    pub station_id: Option<u32>,
    pub abilities: Abilities,
    pub moves: Vec<Move>,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub players: Vec<PlayerGame>,
    pub mister_x: MisterXGame,
}
