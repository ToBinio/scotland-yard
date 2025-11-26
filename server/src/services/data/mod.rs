use std::sync::Arc;

use serde::Serialize;

pub mod service;

#[derive(Serialize, Clone)]
pub struct Round {
    pub index: u8,
    pub show_mister_x: bool,
}

#[derive(Serialize, Clone)]
pub struct Connection {
    pub from: u8,
    pub to: u8,
    pub mode: StationType,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum StationType {
    Taxi,
    Bus,
    Underground,
    Water,
}

#[derive(Serialize, Clone)]
pub struct Station {
    pub id: u8,
    pub pos_x: u32,
    pub pos_y: u32,
    pub types: Vec<StationType>,
}

pub type DataServiceHandle = Arc<dyn DataServiceTrait>;

pub trait DataServiceTrait: Send + Sync {
    fn get_all_stations(&self) -> Vec<Station>;
    fn get_all_connections(&self) -> Vec<Connection>;
    fn get_all_rounds(&self) -> Vec<Round>;
    fn get_colors(&self) -> [&str; 5];
    fn get_random_stations(&self, count: usize) -> Vec<u8>;
}
