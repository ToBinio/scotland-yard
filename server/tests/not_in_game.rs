use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::start_game_with_colors,
    test_server,
    ws::{assert_receive_error, assert_receive_message, get_ws_connection, send_message},
};

mod common;

#[tokio::test]
async fn sends_error() {
    let server = test_server();
    let mut player = get_ws_connection(&server).await;

    send_message(&mut player, "startGame", None).await;
    assert_receive_error(&mut player, "not in lobby").await;

    send_message(
        &mut player,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "hidden" }])),
    )
    .await;
    assert_receive_error(&mut player, "not in game").await;

    send_message(
        &mut player,
        "moveDetective",
        Some(json!({ "color": "red", "station_id": 109, "transport_type": "underground" })),
    )
    .await;
    assert_receive_error(&mut player, "not in game").await;

    send_message(&mut player, "submitMove", None).await;
    assert_receive_error(&mut player, "not in game").await;
}

#[derive(Debug, Deserialize)]
struct EndMove;

#[tokio::test]
async fn sends_error_after_finished_game() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    game.full_move_mister_x(110).await;

    let _ = game.send_detective_move(&colors[0], 110, "taxi").await;
    let _ = game.send_detective_move(&colors[1], 107, "bus").await;
    let _ = game.send_detective_move(&colors[2], 108, "bus").await;
    let _ = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_game_ended_message("detective").await;

    // After lose

    send_message(&mut game.detective, "startGame", None).await;
    assert_receive_error(&mut game.detective, "not in lobby").await;

    send_message(&mut game.detective, "submitMove", None).await;
    assert_receive_error(&mut game.detective, "not in game").await;
}
