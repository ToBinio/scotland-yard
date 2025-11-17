use std::sync::LazyLock;

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/stations", get(get_all_stations))
        .route("/connections", get(get_all_connections))
        .with_state(state)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
enum StationType {
    Taxi,
    Bus,
    Underground,
    Water,
}

#[derive(Serialize, Clone)]
struct Station {
    id: u8,
    pos_x: u32,
    pos_y: u32,
    types: Vec<StationType>,
}

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

async fn get_all_stations() -> Json<Vec<Station>> {
    Json((*STATIONS).clone())
}

#[derive(Serialize, Clone)]
struct Connection {
    from: u8,
    to: u8,
    mode: StationType,
}

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

async fn get_all_connections() -> Json<Vec<Connection>> {
    Json((*CONNECTIONS).clone())
}
