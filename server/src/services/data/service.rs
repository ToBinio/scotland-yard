use std::sync::LazyLock;

use game::data::StationType;
use rand::seq::SliceRandom;

use crate::services::data::{Connection, DataServiceTrait, Round, Station};

static STATIONS: LazyLock<Vec<Station>> = LazyLock::new(|| {
    let string = include_str!("../../../data/stations.txt");
    string
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split(' ').collect();
            Station {
                id: parts[0].parse().unwrap(),
                pos_x: parts[1].parse().unwrap(),
                pos_y: parts[2].parse().unwrap(),
                types: parts[3]
                    .split(",")
                    .map(|t| match t {
                        "taxi" => StationType::Taxi,
                        "bus" => StationType::Bus,
                        "underground" => StationType::Underground,
                        _ => panic!("Invalid station type"),
                    })
                    .collect(),
            }
        })
        .collect()
});

static CONNECTIONS: LazyLock<Vec<Connection>> = LazyLock::new(|| {
    let string = include_str!("../../../data/connections.txt");
    string
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split(' ').collect();
            Connection {
                from: parts[0].parse().unwrap(),
                to: parts[1].parse().unwrap(),
                mode: match parts[2] {
                    "taxi" => StationType::Taxi,
                    "bus" => StationType::Bus,
                    "underground" => StationType::Underground,
                    "water" => StationType::Water,
                    _ => panic!("Invalid station type"),
                },
            }
        })
        .collect()
});

#[derive(Default)]
pub struct DataService;

impl DataServiceTrait for DataService {
    fn get_all_stations(&self) -> Vec<Station> {
        STATIONS.clone()
    }

    fn get_all_connections(&self) -> Vec<Connection> {
        CONNECTIONS.clone()
    }

    fn get_all_rounds(&self) -> Vec<Round> {
        const SHOWS_MISTER_X: [i32; 5] = [3, 8, 13, 18, 24];

        (1..=24)
            .map(|index| Round {
                index: index as u8,
                show_mister_x: SHOWS_MISTER_X.contains(&{ index }),
            })
            .collect()
    }

    fn get_colors(&self) -> [&str; 5] {
        ["red", "blue", "green", "yellow", "purple"]
    }

    fn get_random_stations(&self, count: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        let mut stations = self.get_all_stations();
        stations.shuffle(&mut rng);
        stations
            .into_iter()
            .take(count)
            .map(|station| station.id)
            .collect()
    }
}
