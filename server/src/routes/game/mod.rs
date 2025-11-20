use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use tokio::sync::mpsc::{self};

use crate::{
    AppState,
    routes::game::packet::{ClientPacket, ErrorPacket, GamePacket, ServerPacket},
    services::lobby::{LobbyId, LobbyServiceHandle, PlayerId},
};

use futures_util::{sink::SinkExt, stream::StreamExt};

pub mod packet;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(lobby_service): State<LobbyServiceHandle>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, lobby_service))
}
async fn handle_socket(socket: WebSocket, lobby_service: LobbyServiceHandle) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let (tx, mut rx) = mpsc::channel(16);

    let mut send_task = tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            let msg = ServerPacket::to_string(&packet);
            println!("msg - {}", msg);
            let _ = ws_sender.send(Message::text(msg)).await;
        }
    });

    let mut recv_task = {
        tokio::spawn(async move {
            let mut player_id: Option<PlayerId> = None;
            let mut lobby_id: Option<LobbyId> = None;

            while let Some(Ok(msg)) = ws_receiver.next().await {
                let packet = match msg {
                    Message::Text(t) => ClientPacket::from(t.as_str()),
                    Message::Close(_) => {
                        break;
                    }
                    _ => {
                        continue;
                    }
                };

                let packet = match packet {
                    Ok(packet) => packet,
                    Err(err) => {
                        tx.send(ServerPacket::Error(ErrorPacket {
                            message: err.to_string(),
                        }))
                        .await
                        .unwrap();

                        continue;
                    }
                };

                match packet {
                    ClientPacket::CreateGame(packet) => {
                        let id = lobby_service
                            .lock()
                            .await
                            .create(packet.number_of_detectives);
                        tx.send(ServerPacket::Game(GamePacket { id }))
                            .await
                            .unwrap();
                    }
                    ClientPacket::JoinGame(packet) => {
                        if player_id.is_some() {
                            tx.send(ServerPacket::Error(ErrorPacket {
                                message: "game already joined".to_string(),
                            }))
                            .await
                            .unwrap();

                            continue;
                        }

                        let result = lobby_service.lock().await.join(packet.id, tx.clone());

                        match result {
                            Ok(id) => {
                                lobby_id = Some(packet.id);
                                player_id = Some(id);
                            }
                            Err(_) => tx
                                .send(ServerPacket::Error(ErrorPacket {
                                    message: "game does not exist".to_string(),
                                }))
                                .await
                                .unwrap(),
                        }
                    }
                    ClientPacket::StartGame => {
                        let result = lobby_service.lock().await.start(lobby_id.unwrap()).await;

                        if result.is_err() {
                            tx.send(ServerPacket::Error(ErrorPacket {
                                message: "game does not have enough players".to_string(),
                            }))
                            .await
                            .unwrap()
                        }
                    }
                }
            }
        })
    };

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
