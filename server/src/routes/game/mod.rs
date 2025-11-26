use std::ops::Not;

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use thiserror::Error;
use tokio::sync::mpsc::{self, Sender};

use crate::{
    AppState,
    game::{Game, GameError, Role},
    routes::game::packet::{ClientPacket, GamePacket, ServerPacket},
    services::{
        game::{GameServiceError, GameServiceHandle},
        lobby::{LobbyId, LobbyServiceError, LobbyServiceHandle, PlayerId},
    },
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
    State(game_service): State<GameServiceHandle>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, lobby_service, game_service))
}
async fn handle_socket(
    socket: WebSocket,
    lobby_service: LobbyServiceHandle,
    game_service: GameServiceHandle,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let (tx, mut rx) = mpsc::channel(16);

    let mut send_task = tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            let msg = ServerPacket::to_string(&packet);
            let _ = ws_sender.send(Message::text(msg)).await;
        }
    });

    let mut recv_task = {
        tokio::spawn(async move {
            let mut connection = Connection {
                player_id: None,
                lobby_id: None,
                lobby_service,
                game_service,
                sender: tx.clone(),
            };

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
                        tx.send(ServerPacket::from_error(err)).await.unwrap();
                        continue;
                    }
                };

                if let Err(err) = connection.handle_client_packet(packet).await {
                    connection.send(ServerPacket::from_error(err)).await;
                }
            }
        })
    };

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error(transparent)]
    Lobby(#[from] LobbyServiceError),

    #[error(transparent)]
    GameService(#[from] GameServiceError),

    #[error(transparent)]
    Game(#[from] GameError),

    #[error("game already joined")]
    GameAlreadyJoined,

    #[error("not your turn")]
    NotAllowedForUser,
}

struct Connection {
    player_id: Option<PlayerId>,
    lobby_id: Option<LobbyId>,

    lobby_service: LobbyServiceHandle,
    game_service: GameServiceHandle,

    sender: Sender<ServerPacket>,
}

impl Connection {
    async fn send(&mut self, packet: ServerPacket) {
        self.sender.send(packet).await.unwrap();
    }

    fn is_own_round(&self, game: &Game) -> Result<(), ConnectionError> {
        if game
            .get_user_role(self.player_id.unwrap())
            .eq(game.active_role())
            .not()
        {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    fn is_detective(&self, game: &Game) -> Result<(), ConnectionError> {
        if matches!(game.get_user_role(self.player_id.unwrap()), Role::Detective) {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    fn is_mister_x(&self, game: &Game) -> Result<(), ConnectionError> {
        if matches!(game.get_user_role(self.player_id.unwrap()), Role::MisterX) {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    async fn handle_client_packet(&mut self, packet: ClientPacket) -> Result<(), ConnectionError> {
        match packet {
            ClientPacket::CreateGame(packet) => {
                let id = self
                    .lobby_service
                    .lock()
                    .await
                    .create(packet.number_of_detectives);
                self.send(ServerPacket::Game(GamePacket { id })).await;
            }
            ClientPacket::JoinGame(packet) => {
                if self.player_id.is_some() {
                    return Err(ConnectionError::GameAlreadyJoined);
                }

                let id = self
                    .lobby_service
                    .lock()
                    .await
                    .join(&packet.id, self.sender.clone())?;

                self.lobby_id = Some(packet.id);
                self.player_id = Some(id);
            }
            ClientPacket::StartGame => {
                let ref_lobby_service = self.lobby_service.lock().await;
                let lobby = ref_lobby_service.get_lobby(&self.lobby_id.unwrap());

                self.game_service
                    .lock()
                    .await
                    .add_game_from_lobby(lobby.unwrap(), &self.lobby_id.unwrap())?;

                drop(ref_lobby_service);

                self.lobby_service
                    .lock()
                    .await
                    .close_lobby(&self.lobby_id.unwrap());

                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.lobby_id.unwrap())?;
                game.start().await;
            }
            ClientPacket::MoveMisterX(packet) => {
                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.lobby_id.unwrap())?;

                self.is_own_round(game)?;
                self.is_detective(game)?;

                game.move_mister_x(
                    packet
                        .into_iter()
                        .map(|packet| (packet.station_id, packet.transport_type))
                        .collect(),
                )?;
            }
            ClientPacket::MoveDetective(packet) => {
                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.lobby_id.unwrap())?;

                self.is_own_round(game)?;
                self.is_mister_x(game)?;

                game.move_detective(packet.color, packet.station_id, packet.transport_type)
                    .await?;
            }
            ClientPacket::SubmitMove => {
                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.lobby_id.unwrap())?;

                self.is_own_round(game)?;

                game.end_move().await?;
            }
        }

        Ok(())
    }
}
