use axum_test::TestWebSocket;
use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::start_game,
    data::{Game, Move},
    test_server,
    ws::{assert_receive_error, assert_receive_message, send_message},
};

mod common;

#[derive(Debug, Deserialize)]
struct EndMove;

async fn receive_start_move_message(player: &mut TestWebSocket, expected_role: &str) {
    #[derive(Debug, Deserialize)]
    struct StartMove {
        role: String,
    }

    let message = assert_receive_message::<StartMove>(player, "startMove").await;
    assert_eq!(message.unwrap().role, expected_role);
}

async fn send_detective_move(
    mister_x: &mut TestWebSocket,
    detective: &mut TestWebSocket,
    message: serde_json::Value,
) -> Game {
    send_message(detective, "moveDetective", Some(message)).await;

    assert_receive_message::<Game>(mister_x, "gameState").await;
    assert_receive_message::<Game>(detective, "gameState")
        .await
        .unwrap()
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

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState").await;

    assert_eq!(game_state.unwrap().mister_x.moves, vec![Move::Taxi]);

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" }),
    )
    .await;

    assert_eq!(game_state.players[0].station_id, 110);
    assert_eq!(game_state.players[0].available_transport.taxi, 9);

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[1], "station_id": 120, "transport_type": "bus" }),
    )
    .await;

    assert_eq!(game_state.players[1].station_id, 120);
    assert_eq!(game_state.players[1].available_transport.bus, 7);

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[2], "station_id": 130, "transport_type": "bus" }),
    )
    .await;

    assert_eq!(game_state.players[2].station_id, 130);
    assert_eq!(game_state.players[2].available_transport.bus, 7);

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[3], "station_id": 140, "transport_type": "underground" }),
    )
    .await;

    assert_eq!(game_state.players[3].station_id, 140);
    assert_eq!(game_state.players[3].available_transport.underground, 3);

    send_message(&mut detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;
}

#[tokio::test]
async fn non_active_can_not_send_or_submit_move() {
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
        &mut detective,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    assert_receive_error(&mut detective, "not your turn").await;

    send_message(
        &mut detective,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" })),
    )
    .await;

    assert_receive_error(&mut detective, "not your turn").await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut detective, "submitMove", None).await;
    assert_receive_error(&mut detective, "not your turn").await;

    send_message(&mut mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut detective, "gameState").await;

    send_message(
        &mut mister_x,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" })),
    )
    .await;
    assert_receive_error(&mut mister_x, "not your turn").await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    assert_receive_error(&mut mister_x, "not your turn").await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" }),
    )
    .await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[1], "station_id": 120, "transport_type": "bus" }),
    )
    .await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[2], "station_id": 130, "transport_type": "bus" }),
    )
    .await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[3], "station_id": 140, "transport_type": "underground" }),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;
    assert_receive_error(&mut mister_x, "not your turn").await;

    send_message(&mut detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;
}

#[tokio::test]
async fn can_only_submit_if_all_moved() {
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

    send_message(&mut mister_x, "submitMove", None).await;
    assert_receive_error(&mut mister_x, "not all moved").await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut detective, "gameState").await;

    send_message(&mut detective, "submitMove", None).await;
    assert_receive_error(&mut detective, "not all moved").await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" }),
    )
    .await;

    send_message(&mut detective, "submitMove", None).await;
    assert_receive_error(&mut detective, "not all moved").await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[1], "station_id": 120, "transport_type": "bus" }),
    )
    .await;

    send_message(&mut detective, "submitMove", None).await;
    assert_receive_error(&mut detective, "not all moved").await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[2], "station_id": 130, "transport_type": "bus" }),
    )
    .await;

    send_message(&mut detective, "submitMove", None).await;
    assert_receive_error(&mut detective, "not all moved").await;

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[3], "station_id": 140, "transport_type": "underground" }),
    )
    .await;

    send_message(&mut detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;
}

#[tokio::test]
async fn can_change_move() {
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
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState").await;

    assert_eq!(game_state.unwrap().mister_x.moves, vec![Move::Taxi]);

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[0], "station_id": 120, "transport_type": "taxi" }),
    )
    .await;

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[0], "station_id": 110, "transport_type": "taxi" }),
    )
    .await;

    assert_eq!(game_state.players[0].station_id, 110);
    assert_eq!(game_state.players[0].available_transport.taxi, 9);

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[1], "station_id": 130, "transport_type": "bus" }),
    )
    .await;

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[1], "station_id": 120, "transport_type": "bus" }),
    )
    .await;

    assert_eq!(game_state.players[1].station_id, 120);
    assert_eq!(game_state.players[1].available_transport.bus, 7);

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[2], "station_id": 140, "transport_type": "bus" }),
    )
    .await;

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[2], "station_id": 130, "transport_type": "bus" }),
    )
    .await;

    assert_eq!(game_state.players[2].station_id, 130);
    assert_eq!(game_state.players[2].available_transport.bus, 7);

    let _ = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[3], "station_id": 150, "transport_type": "underground" }),
    )
    .await;

    let game_state = send_detective_move(
        &mut mister_x,
        &mut detective,
        json!({ "color": colors[3], "station_id": 140, "transport_type": "underground" }),
    )
    .await;

    assert_eq!(game_state.players[3].station_id, 140);
    assert_eq!(game_state.players[3].available_transport.underground, 3);

    send_message(&mut detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;
}

#[tokio::test]
async fn can_double_move() {
    let mut server = test_server();
    let (mut mister_x, mut detective) = start_game(&mut server).await;

    receive_start_move_message(&mut mister_x, "mister_x").await;
    receive_start_move_message(&mut detective, "mister_x").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let _ = assert_receive_message::<Game>(&mut detective, "gameState").await;

    send_message(&mut mister_x, "moveMisterX", Some(json!([]))).await;
    assert_receive_error(&mut mister_x, "invalid move").await;

    send_message(&mut mister_x, "moveMisterX", Some(json!([{ "station_id": 100, "transport_type": "taxi" },{ "station_id": 100, "transport_type": "taxi" },{ "station_id": 100, "transport_type": "taxi" }]))).await;
    assert_receive_error(&mut mister_x, "invalid move").await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "taxi" },{ "station_id": 110, "transport_type": "bus" }])),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.mister_x.moves, vec![Move::Taxi, Move::Bus]);
    assert_eq!(game_state.mister_x.abilities.double_move, 1);
}

#[tokio::test]
async fn can_move_hidden() {
    let mut server = test_server();
    let (mut mister_x, mut detective) = start_game(&mut server).await;

    receive_start_move_message(&mut mister_x, "mister_x").await;
    receive_start_move_message(&mut detective, "mister_x").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let _ = assert_receive_message::<Game>(&mut detective, "gameState").await;

    send_message(
        &mut mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 100, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut detective, "endMove").await;

    receive_start_move_message(&mut mister_x, "detective").await;
    receive_start_move_message(&mut detective, "detective").await;

    assert_receive_message::<Game>(&mut mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.mister_x.moves, vec![Move::Hidden]);
    assert_eq!(game_state.mister_x.abilities.hidden, 1);
}
