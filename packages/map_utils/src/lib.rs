use game::data::{Connection, StationType};

pub fn all_taxi_connections(connections: &[Connection], station: u8) -> Vec<u8> {
    connections
        .iter()
        .filter(|c| c.from == station || c.to == station)
        .filter(|c| c.mode == StationType::Taxi)
        .map(|c| if c.from == station { c.to } else { c.from })
        .collect()
}
