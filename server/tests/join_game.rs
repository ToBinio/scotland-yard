use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::create_game,
    test_server,
    ws::{assert_receive_error, assert_receive_message, get_ws_connection, send_message},
};

mod common;

#[tokio::test]
async fn can_start_game() {
    let (server, _dir) = test_server();

    let mut player_1 = get_ws_connection(&server).await;
    let mut player_2 = get_ws_connection(&server).await;

    let game_id = create_game(&mut player_1).await;

    send_message(&mut player_1, "joinGame", Some(json!({ "id": game_id }))).await;
    send_message(&mut player_2, "joinGame", Some(json!({ "id": game_id }))).await;

    send_message(&mut player_2, "startGame", None).await;

    #[derive(Debug, Deserialize)]
    struct GameStarted {
        role: String,
    }

    let response = assert_receive_message::<GameStarted>(&mut player_1, "gameStarted").await;
    let role_1 = response.unwrap().role;

    let response = assert_receive_message::<GameStarted>(&mut player_2, "gameStarted").await;
    let role_2 = response.unwrap().role;

    if role_1 == "detective" {
        assert_eq!(role_1, "detective");
        assert_eq!(role_2, "mister_x");
    } else {
        assert_eq!(role_1, "mister_x");
        assert_eq!(role_2, "detective");
    }
}

#[tokio::test]
async fn can_not_join_game_twice() {
    let (server, _dir) = test_server();

    let mut player = get_ws_connection(&server).await;
    let game_id = create_game(&mut player).await;

    send_message(&mut player, "joinGame", Some(json!({ "id": game_id }))).await;
    send_message(&mut player, "joinGame", Some(json!({ "id": game_id }))).await;

    assert_receive_error(&mut player, "game already joined").await;
}

#[tokio::test]
async fn can_not_join_unknown_game() {
    let (server, _dir) = test_server();

    let mut player = get_ws_connection(&server).await;

    send_message(
        &mut player,
        "joinGame",
        Some(json!({ "id": "fffdc005-f76c-49d1-b39a-cbbb801eaece" })),
    )
    .await;

    assert_receive_error(&mut player, "unknown lobby").await;
}

#[tokio::test]
async fn can_not_start_game_without_enough_players() {
    let (server, _dir) = test_server();

    let mut player = get_ws_connection(&server).await;
    let game_id = create_game(&mut player).await;

    send_message(&mut player, "joinGame", Some(json!({ "id": game_id }))).await;

    send_message(&mut player, "startGame", None).await;

    assert_receive_error(&mut player, "game does not have enough players").await;
}

#[tokio::test]
async fn can_not_join_started_game() {
    let (server, _dir) = test_server();

    let mut player_1 = get_ws_connection(&server).await;
    let mut player_2 = get_ws_connection(&server).await;
    let mut player_3 = get_ws_connection(&server).await;

    let game_id = create_game(&mut player_1).await;

    send_message(&mut player_1, "joinGame", Some(json!({ "id": game_id }))).await;
    send_message(&mut player_2, "joinGame", Some(json!({ "id": game_id }))).await;

    send_message(&mut player_2, "startGame", None).await;

    #[derive(Debug, Deserialize)]
    struct GameStarted {}

    let _ = assert_receive_message::<GameStarted>(&mut player_1, "gameStarted").await;

    let _ = assert_receive_message::<GameStarted>(&mut player_2, "gameStarted").await;

    send_message(&mut player_3, "joinGame", Some(json!({ "id": game_id }))).await;
    assert_receive_error(&mut player_3, "unknown lobby").await;
}
