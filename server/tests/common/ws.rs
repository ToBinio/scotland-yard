use std::time::Duration;

use axum_test::{TestServer, TestWebSocket};
use serde::Deserialize;
use tokio::time::timeout;

pub async fn get_ws_connection(server: &TestServer) -> TestWebSocket {
    server
        .get_websocket("/game/ws")
        .await
        .into_websocket()
        .await
}

pub async fn send_message(
    connection: &mut TestWebSocket,
    name: &str,
    message: Option<serde_json::Value>,
) {
    if let Some(message) = message {
        connection
            .send_text(format!("[{}] {}", name, message))
            .await;
    } else {
        connection.send_text(format!("[{}]", name)).await;
    }
}

pub async fn receive_message<T: serde::de::DeserializeOwned>(
    connection: &mut TestWebSocket,
) -> (String, Option<T>) {
    let message = timeout(Duration::from_millis(500), connection.receive_text())
        .await
        .unwrap();
    let mut split = message.splitn(2, " ");

    let name = split
        .next()
        .unwrap()
        .trim_matches('[')
        .trim_end_matches(']')
        .to_string();

    if let Some(data) = split.next() {
        let data = serde_json::from_str(data).unwrap();
        return (name, Some(data));
    }

    (name, None)
}

pub async fn assert_receive_message<T: serde::de::DeserializeOwned>(
    connection: &mut TestWebSocket,
    name: &str,
) -> Option<T> {
    let (received_name, message) = timeout(Duration::from_millis(200), receive_message(connection))
        .await
        .map_err(|_| format!("expected ws packet '{}' but did not recieve", name))
        .unwrap();
    assert_eq!(received_name, name);
    message
}

pub async fn assert_receive_error(connection: &mut TestWebSocket, message: &str) {
    #[derive(Debug, Deserialize)]
    struct Error {
        message: String,
    }

    let (received_name, response) = receive_message::<Error>(connection).await;
    assert_eq!(received_name, "error");
    assert_eq!(response.unwrap().message, message);
}
