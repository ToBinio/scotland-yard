use crate::common::test_server;
use serde::Deserialize;

mod common;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum StationType {
    Taxi,
    Bus,
    Underground,
    Water,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Connection {
    from: u8,
    to: u8,
    mode: StationType,
}

#[tokio::test]
async fn get_connections() {
    let server = test_server();

    let response = server.get("/connections").await;

    response.assert_status_ok();
    let response = response.json::<Vec<Connection>>();

    assert_eq!(response.len(), 468);

    assert!(response.iter().any(|connection| {
        connection.eq(&Connection {
            from: 108,
            to: 115,
            mode: StationType::Water,
        })
    }));

    assert!(response.iter().any(|connection| {
        connection.eq(&Connection {
            from: 89,
            to: 140,
            mode: StationType::Underground,
        })
    }));

    assert!(response.iter().any(|connection| {
        connection.eq(&Connection {
            from: 184,
            to: 185,
            mode: StationType::Bus,
        })
    }));

    assert!(response.iter().any(|connection| {
        connection.eq(&Connection {
            from: 42,
            to: 72,
            mode: StationType::Taxi,
        })
    }));
}
