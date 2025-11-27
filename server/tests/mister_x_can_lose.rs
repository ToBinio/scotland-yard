use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::start_game,
    data::Game,
    test_server,
    ws::{assert_receive_message, send_message},
};

mod common;

#[derive(Debug, Deserialize)]
struct EndMove;

#[tokio::test]
async fn can_lose_after_detective_move() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;
    let colors: Vec<_> = game_state
        .unwrap()
        .players
        .iter()
        .map(|player| player.color.clone())
        .collect();

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

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
}

#[tokio::test]
async fn can_lose_after_mister_x_move() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;
    let colors: Vec<_> = game_state
        .unwrap()
        .players
        .iter()
        .map(|player| player.color.clone())
        .collect();

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "hidden" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("detective").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    let _ = game.send_detective_move(&colors[0], 106, "taxi").await;
    let _ = game.send_detective_move(&colors[1], 107, "bus").await;
    let _ = game.send_detective_move(&colors[2], 108, "bus").await;
    let _ = game
        .send_detective_move(&colors[3], 109, "underground")
        .await;

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert_receive_message::<Game>(&mut game.detective, "gameState").await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 106, "transport_type": "bus" }])),
    )
    .await;

    send_message(&mut game.mister_x, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_game_ended_message("detective").await;
}
