use axum_test::TestWebSocket;
use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::start_game,
    data::{Game, Move},
    test_server,
    ws::{assert_receive_message, send_message},
};

mod common;

async fn receive_start_move_message(mut player: &mut TestWebSocket, expected_role: &str) {
    #[derive(Debug, Deserialize)]
    struct StartMove {
        role: String,
    }

    let message = assert_receive_message::<StartMove>(&mut player, "startMove").await;
    assert_eq!(message.unwrap().role, expected_role);
}

#[tokio::test]
async fn can_move() {
    let mut server = test_server();
    let (mut mister_x, mut detective) = start_game(&mut server).await;

    receive_start_move_message(&mut mister_x, "mister_x").await;
    receive_start_move_message(&mut detective, "mister_x").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState").await;
    let colors: Vec<_> = game_state
        .unwrap()
        .players
        .iter()
        .map(|player| player.color.clone())
        .collect();

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;

    #[derive(Debug, Deserialize)]
    struct EndMove;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState").await;

    assert_eq!(game_state.unwrap().mister_x.moves, vec![Move::Taxi]);

    send_message(
        &mut detective,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" })),
    )
    .await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.players[0].station_id, 110);
    assert_eq!(game_state.players[0].available_transport.taxi, 9);

    send_message(
        &mut detective,
        "moveDetective",
        Some(json!({ "color": colors[1], "station_id": 120, "transport_type": "bus" })),
    )
    .await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.players[1].station_id, 120);
    assert_eq!(game_state.players[1].available_transport.bus, 7);

    send_message(
        &mut detective,
        "moveDetective",
        Some(json!({ "color": colors[2], "station_id": 130, "transport_type": "bus" })),
    )
    .await;

    assert_eq!(game_state.players[2].station_id, 130);
    assert_eq!(game_state.players[2].available_transport.bus, 7);

    send_message(
        &mut detective,
        "moveDetective",
        Some(json!({ "color": colors[3], "station_id": 140, "transport_type": "underground" })),
    )
    .await;

    assert_eq!(game_state.players[3].station_id, 140);
    assert_eq!(game_state.players[3].available_transport.underground, 3);

    send_message(&mut detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;
}
