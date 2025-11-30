#![allow(dead_code)]

use std::{sync::Arc, vec};

use axum_test::TestServer;
use game::data::{Connection, Round, Station, StationType};
use server::{
    app,
    services::data::{
        DataServiceTrait,
        service::{self},
    },
};

pub mod connection;
pub mod data;
pub mod ws;

pub fn test_server() -> TestServer {
    let app = app(Arc::new(DataService));
    TestServer::builder().http_transport().build(app).unwrap()
}

pub fn test_prod_server() -> TestServer {
    let app = app(Arc::new(service::DataService));
    TestServer::builder().http_transport().build(app).unwrap()
}

struct DataService;

impl DataServiceTrait for DataService {
    fn get_all_stations(&self) -> Vec<Station> {
        let mut stations = vec![];

        for i in 0..25 {
            stations.push(Station {
                id: 100 + i,
                pos_x: 0,
                pos_y: 0,
                types: vec![
                    StationType::Taxi,
                    StationType::Bus,
                    StationType::Underground,
                    StationType::Water,
                ],
            });
        }

        stations
    }

    fn get_all_connections(&self) -> Vec<Connection> {
        vec![
            //player1 - red
            Connection {
                from: 100,
                to: 106,
                mode: StationType::Taxi,
            },
            Connection {
                from: 100,
                to: 110,
                mode: StationType::Taxi,
            },
            Connection {
                from: 100,
                to: 116,
                mode: StationType::Taxi,
            },
            //player2 - blue
            Connection {
                from: 101,
                to: 107,
                mode: StationType::Bus,
            },
            Connection {
                from: 101,
                to: 117,
                mode: StationType::Bus,
            },
            //player3 - green
            Connection {
                from: 102,
                to: 108,
                mode: StationType::Bus,
            },
            Connection {
                from: 102,
                to: 118,
                mode: StationType::Bus,
            },
            //player4 - yellow
            Connection {
                from: 103,
                to: 109,
                mode: StationType::Underground,
            },
            Connection {
                from: 103,
                to: 109,
                mode: StationType::Taxi,
            },
            Connection {
                from: 103,
                to: 119,
                mode: StationType::Underground,
            },
            //mister-x
            Connection {
                from: 104,
                to: 110,
                mode: StationType::Taxi,
            },
            Connection {
                from: 104,
                to: 120,
                mode: StationType::Water,
            },
            Connection {
                from: 110,
                to: 120,
                mode: StationType::Bus,
            },
            Connection {
                from: 110,
                to: 106,
                mode: StationType::Bus,
            },
        ]
    }

    fn get_all_rounds(&self) -> Vec<Round> {
        const SHOWS_MISTER_X: [i32; 5] = [3, 8, 13, 18, 24];

        (1..=5)
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
        let mut stations = vec![];

        for i in 0..(count as u8) {
            stations.push(100 + i);
        }

        stations
    }
}
