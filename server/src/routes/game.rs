use axum::{
    Router,
    extract::{WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::any,
};

use crate::AppState;

use futures_util::stream::StreamExt;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}
async fn handle_socket(socket: WebSocket) {
    let (_sender, _receiver) = socket.split();
}
