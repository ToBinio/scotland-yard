use std::sync::Arc;

use game::data::{Connection, Round, Station};

pub mod service;

pub type DataServiceHandle = Arc<dyn DataServiceTrait>;

pub trait DataServiceTrait: Send + Sync {
    fn get_all_stations(&self) -> Vec<Station>;
    fn get_all_connections(&self) -> Vec<Connection>;
    fn get_all_rounds(&self) -> Vec<Round>;
    fn get_colors(&self) -> [&str; 5];
    fn get_random_stations(&self, count: usize) -> Vec<u8>;
}
