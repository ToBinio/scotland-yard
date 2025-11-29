use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::{start_game, start_game_with_colors},
    data::{Game, Move},
    test_server,
    ws::{assert_receive_error, assert_receive_message, send_message},
};

mod common;

#[derive(Debug, Deserialize)]
struct EndMove;

#[tokio::test]
async fn can_move() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    assert_eq!(game_state.unwrap().mister_x.moves, vec![Move::Taxi]);

    let game_state = game.send_detective_move(&colors[0], 106, "taxi").await;

    assert_eq!(game_state.players[0].station_id, 106);
    assert_eq!(game_state.players[0].available_transport.taxi, 9);

    let game_state = game.send_detective_move(&colors[1], 107, "bus").await;

    assert_eq!(game_state.players[1].station_id, 107);
    assert_eq!(game_state.players[1].available_transport.bus, 7);

    let game_state = game.send_detective_move(&colors[2], 108, "bus").await;

    assert_eq!(game_state.players[2].station_id, 108);
    assert_eq!(game_state.players[2].available_transport.bus, 7);

    let game_state = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    assert_eq!(game_state.players[3].station_id, 109);
    assert_eq!(game_state.players[3].available_transport.underground, 3);

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;
}

#[tokio::test]
async fn non_active_can_not_send_or_submit_move() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    send_message(
        &mut game.detective,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    assert_receive_error(&mut game.detective, "not your turn").await;

    send_message(
        &mut game.detective,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 106, "transport_type": "taxi" })),
    )
    .await;

    assert_receive_error(&mut game.detective, "not your turn").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not your turn").await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(
        &mut game.mister_x,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 106, "transport_type": "taxi" })),
    )
    .await;
    assert_receive_error(&mut game.mister_x, "not your turn").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    assert_receive_error(&mut game.mister_x, "not your turn").await;

    let _ = game.send_detective_move(&colors[0], 106, "taxi").await;
    let _ = game.send_detective_move(&colors[1], 107, "bus").await;
    let _ = game.send_detective_move(&colors[2], 108, "bus").await;
    let _ = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    send_message(&mut game.mister_x, "submitMove", None).await;
    assert_receive_error(&mut game.mister_x, "not your turn").await;

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;
}

#[tokio::test]
async fn can_only_submit_if_all_moved() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    send_message(&mut game.mister_x, "submitMove", None).await;
    assert_receive_error(&mut game.mister_x, "not all moved").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not all moved").await;

    let _ = game.send_detective_move(&colors[0], 106, "taxi").await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not all moved").await;

    let _ = game.send_detective_move(&colors[1], 107, "bus").await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not all moved").await;

    let _ = game.send_detective_move(&colors[2], 108, "bus").await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not all moved").await;

    let _ = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;
}

#[tokio::test]
async fn can_change_move() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 120, "transport_type": "hidden" }])),
    )
    .await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    assert_eq!(game_state.unwrap().mister_x.moves, vec![Move::Taxi]);

    let _ = game.send_detective_move(&colors[0], 116, "taxi").await;

    let game_state = game.send_detective_move(&colors[0], 106, "taxi").await;

    assert_eq!(game_state.players[0].station_id, 106);
    assert_eq!(game_state.players[0].available_transport.taxi, 9);

    let _ = game.send_detective_move(&colors[1], 117, "bus").await;

    let game_state = game.send_detective_move(&colors[1], 107, "bus").await;

    assert_eq!(game_state.players[1].station_id, 107);
    assert_eq!(game_state.players[1].available_transport.bus, 7);

    let _ = game.send_detective_move(&colors[2], 108, "bus").await;

    let game_state = game.send_detective_move(&colors[2], 118, "bus").await;

    assert_eq!(game_state.players[2].station_id, 118);
    assert_eq!(game_state.players[2].available_transport.bus, 7);

    let _ = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    let game_state = game
        .send_detective_move(&colors[3], 119, "underground")
        .await;

    assert_eq!(game_state.players[3].station_id, 119);
    assert_eq!(game_state.players[3].available_transport.underground, 3);

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;
}

#[tokio::test]
async fn can_double_move() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let _ = assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(&mut game.mister_x, "moveMisterX", Some(json!([]))).await;
    assert_receive_error(&mut game.mister_x, "invalid move").await;

    send_message(&mut game.mister_x, "moveMisterX", Some(json!([{ "station_id": 110, "transport_type": "taxi" },{ "station_id": 110, "transport_type": "taxi" },{ "station_id": 110, "transport_type": "taxi" }]))).await;
    assert_receive_error(&mut game.mister_x, "invalid move").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" },{ "station_id": 120, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.mister_x.moves, vec![Move::Taxi, Move::Hidden]);
    assert_eq!(game_state.mister_x.abilities.double_move, 1);
    assert_eq!(game_state.mister_x.abilities.hidden, 1);
}

#[tokio::test]
async fn can_move_hidden() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let _ = assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 120, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState")
        .await
        .unwrap();

    assert_eq!(game_state.mister_x.moves, vec![Move::Hidden]);
    assert_eq!(game_state.mister_x.abilities.hidden, 1);
}

#[tokio::test]
async fn can_only_do_valid_moves() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "bus" }])),
    )
    .await;

    assert_receive_error(&mut game.mister_x, "invalid move").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 106, "transport_type": "taxi" }])),
    )
    .await;

    assert_receive_error(&mut game.mister_x, "invalid move").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not your turn").await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(
        &mut game.detective,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 106, "transport_type": "underground" })),
    )
    .await;
    assert_receive_error(&mut game.detective, "invalid move").await;

    send_message(
        &mut game.detective,
        "moveDetective",
        Some(json!({ "color": colors[0], "station_id": 107, "transport_type": "bus" })),
    )
    .await;
    assert_receive_error(&mut game.detective, "invalid move").await;

    let _ = game.send_detective_move(&colors[0], 106, "taxi").await;
}
