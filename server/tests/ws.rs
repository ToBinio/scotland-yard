use serde_json::json;

use crate::common::{
    test_server,
    ws::{assert_receive_error, get_ws_connection, send_message},
};

mod common;

#[tokio::test]
async fn can_connect() {
    let (server, _dir) = test_server();

    let response = server.get_websocket("/game/ws").await;

    response.assert_status_switching_protocols();
}

#[tokio::test]
async fn handels_unknown_packet() {
    let (server, _dir) = test_server();

    let mut player = get_ws_connection(&server).await;

    send_message(&mut player, "unknown", None).await;
    assert_receive_error(&mut player, "unknown packet").await;
}

#[tokio::test]
async fn handels_invalid_packet() {
    let (server, _dir) = test_server();

    let mut player = get_ws_connection(&server).await;

    send_message(
        &mut player,
        "createGame",
        Some(json!({ "something": "that field does not exist" })),
    )
    .await;
    assert_receive_error(&mut player, "invalid packet").await;
}
