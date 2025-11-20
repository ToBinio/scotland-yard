use axum_test::{TestServer, TestWebSocket};
use serde::Deserialize;
use serde_json::json;

use crate::common::{
    test_server,
    ws::{assert_receive_message, get_ws_connection, send_message},
};

pub async fn create_game(socket: &mut TestWebSocket) -> String {
    send_message(
        socket,
        "createGame",
        Some(json!({
            "number_of_detectives": 4,
        })),
    )
    .await;

    #[derive(Debug, Deserialize)]
    struct GameCreated {
        id: String,
    }

    let response = assert_receive_message::<GameCreated>(socket, "game").await;

    response.unwrap().id
}

pub async fn start_game(server: &mut TestServer) -> (TestWebSocket, TestWebSocket) {
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

    let _ = assert_receive_message::<GameStarted>(&mut player_2, "gameStarted").await;

    if role_1 == "detective" {
        (player_2, player_1)
    } else {
        (player_1, player_2)
    }
}
