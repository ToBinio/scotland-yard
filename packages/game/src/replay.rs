use std::collections::HashMap;

use serde::Serialize;

use crate::{
    character::{detective, mister_x},
    event::Role,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Action {
    Detective {
        color: String,
        action: detective::Action,
    },
    MisterX(mister_x::Action),
}

#[derive(Debug, Clone, Serialize)]
pub struct Replay {
    pub mister_x_starting_station: u8,
    pub detective_starting_stations: HashMap<String, u8>,
    pub actions: Vec<Action>,
    pub winner: Role,
}
