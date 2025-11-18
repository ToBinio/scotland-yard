use crate::common::test_server;
use serde::Deserialize;

mod common;

#[derive(Deserialize, Debug, PartialEq)]
struct Round {
    index: u8,
    show_mister_x: bool,
}

#[tokio::test]
async fn get_rounds() {
    let server = test_server();

    let response = server.get("/map/rounds").await;

    response.assert_status_ok();
    let response = response.json::<Vec<Round>>();

    assert_eq!(response.len(), 24);

    assert!(response.iter().any(|round| {
        round.eq(&Round {
            index: 1,
            show_mister_x: false,
        })
    }));

    assert!(response.iter().any(|round| {
        round.eq(&Round {
            index: 3,
            show_mister_x: true,
        })
    }));

    assert!(response.iter().any(|round| {
        round.eq(&Round {
            index: 13,
            show_mister_x: true,
        })
    }));

    assert!(response.iter().any(|round| {
        round.eq(&Round {
            index: 22,
            show_mister_x: false,
        })
    }));
}
