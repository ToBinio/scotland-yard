use std::sync::LazyLock;

use serde::Serialize;

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
    id: u8,
    pos_x: u32,
    pos_y: u32,
    types: Vec<StationType>,
}

static STATIONS: LazyLock<Vec<Station>> = LazyLock::new(|| {
    let string = include_str!("../../data/stations.txt");
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

#[derive(Serialize, Clone)]
pub struct Connection {
    from: u8,
    to: u8,
    mode: StationType,
}

static CONNECTIONS: LazyLock<Vec<Connection>> = LazyLock::new(|| {
    let string = include_str!("../../data/connections.txt");
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

#[derive(Serialize, Clone)]
pub struct Round {
    index: u8,
    show_mister_x: bool,
}

#[derive(Default)]
pub struct DataService;

impl DataService {
    pub fn get_all_stations(&self) -> Vec<Station> {
        STATIONS.clone()
    }

    pub fn get_all_connections(&self) -> Vec<Connection> {
        CONNECTIONS.clone()
    }

    pub fn get_all_rounds(&self) -> Vec<Round> {
        const SHOWS_MISTER_X: [i32; 5] = [3, 8, 13, 18, 24];

        let rounds = (1..=24)
            .map(|index| Round {
                index: index as u8,
                show_mister_x: SHOWS_MISTER_X.contains(&(index as i32)),
            })
            .collect();

        rounds
    }
}
