use axum_test::TestWebSocket;
use serde::Deserialize;
use serde_json::json;

use crate::common::ws::{assert_receive_message, send_message};

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
