use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Detective,
    MisterX,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MisterXActionType {
    Taxi,
    Bus,
    Underground,
    Hidden,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectiveActionType {
    Taxi,
    Bus,
    Underground,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DetectiveData {
    pub color: String,
    pub station_id: u8,
    pub available_transport: DetectiveTransportData,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DetectiveTransportData {
    pub taxi: u8,
    pub bus: u8,
    pub underground: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MisterXData {
    pub station_id: Option<u8>,
    pub abilities: MisterXAbilityData,
    pub moves: Vec<MisterXActionType>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MisterXAbilityData {
    pub double_move: u8,
    pub hidden: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GameState {
    pub players: Vec<DetectiveData>,
    pub mister_x: MisterXData,
}

#[allow(async_fn_in_trait)]
pub trait EventListener {
    async fn on_game_start(&self);
    async fn on_start_round(&self, role: &Role);
    async fn on_end_move(&self);
    async fn on_game_ended(&self, role: &Role);
    async fn on_game_state_update(&self, state: GameState, show_mister_x: bool);
}
