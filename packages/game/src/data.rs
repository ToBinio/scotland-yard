use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Round {
    pub index: u8,
    pub show_mister_x: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub from: u8,
    pub to: u8,
    pub mode: StationType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StationType {
    Taxi,
    Bus,
    Underground,
    Water,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Station {
    pub id: u8,
    pub pos_x: u32,
    pub pos_y: u32,
    pub types: Vec<StationType>,
}
