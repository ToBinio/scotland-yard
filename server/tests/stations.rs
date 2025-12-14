use crate::common::test_prod_server;
use serde::Deserialize;

mod common;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum StationType {
    Taxi,
    Bus,
    Underground,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Station {
    id: u8,
    pos_x: u32,
    pos_y: u32,
    types: Vec<StationType>,
}

#[tokio::test]
async fn get_stations() {
    let (server, _dir) = test_prod_server();

    let response = server.get("/map/stations").await;

    response.assert_status_ok();
    let response = response.json::<Vec<Station>>();

    assert_eq!(response.len(), 199);

    assert_eq!(
        response.iter().find(|station| station.id == 1).unwrap(),
        &Station {
            id: 1,
            pos_x: 190,
            pos_y: 40,
            types: vec![
                StationType::Taxi,
                StationType::Bus,
                StationType::Underground
            ]
        }
    );

    assert_eq!(
        response.iter().find(|station| station.id == 43).unwrap(),
        &Station {
            id: 43,
            pos_x: 37,
            pos_y: 277,
            types: vec![StationType::Taxi]
        }
    );

    assert_eq!(
        response.iter().find(|station| station.id == 199).unwrap(),
        &Station {
            id: 199,
            pos_x: 1322,
            pos_y: 1186,
            types: vec![StationType::Taxi, StationType::Bus]
        }
    );
}
