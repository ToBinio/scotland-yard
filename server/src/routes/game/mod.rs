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
use uuid::Uuid;

use crate::{
    AppState,
    game::{Game, GameError, Role},
    routes::game::packet::{ClientPacket, GamePacket, ServerPacket},
    services::{
        game::{GameServiceError, GameServiceHandle},
        lobby::{LobbyServiceError, LobbyServiceHandle},
        ws_connection::WsConnectionServiceHandle,
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
    State(ws_connection_service): State<WsConnectionServiceHandle>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| {
        handle_socket(socket, lobby_service, game_service, ws_connection_service)
    })
}
async fn handle_socket(
    socket: WebSocket,
    lobby_service: LobbyServiceHandle,
    game_service: GameServiceHandle,
    ws_connection_service: WsConnectionServiceHandle,
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
            let uuid = Uuid::new_v4();

            ws_connection_service
                .lock()
                .await
                .add_connection(uuid, tx.clone());

            let mut connection = Connection {
                connection_id: uuid,
                lobby_service,
                game_service,
                ws_connection_service,
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

    #[error("not in game")]
    NotInGame,

    #[error("not in lobby")]
    NotInLobby,
}

struct Connection {
    connection_id: Uuid,

    lobby_service: LobbyServiceHandle,
    game_service: GameServiceHandle,
    ws_connection_service: WsConnectionServiceHandle,
}

impl Connection {
    async fn sender(&self) -> Sender<ServerPacket> {
        self.ws_connection_service
            .lock()
            .await
            .ws_sender(self.connection_id)
            .unwrap()
    }

    async fn send(&self, packet: ServerPacket) {
        self.sender().await.send(packet).await.unwrap();
    }

    async fn lobby_id(&self) -> Option<Uuid> {
        self.ws_connection_service
            .lock()
            .await
            .lobby_id(self.connection_id)
            .unwrap()
    }

    async fn set_lobby_id(&self, id: Option<Uuid>) {
        self.ws_connection_service
            .lock()
            .await
            .set_lobby_id(self.connection_id, id)
            .unwrap();
    }

    async fn game_id(&self) -> Option<Uuid> {
        self.ws_connection_service
            .lock()
            .await
            .game_id(self.connection_id)
            .unwrap()
    }

    fn assert_own_round(&self, game: &Game) -> Result<(), ConnectionError> {
        if game
            .get_user_role(self.connection_id)
            .eq(game.active_role())
            .not()
        {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    fn assert_detective(&self, game: &Game) -> Result<(), ConnectionError> {
        if matches!(game.get_user_role(self.connection_id), Role::Detective) {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    fn assert_mister_x(&self, game: &Game) -> Result<(), ConnectionError> {
        if matches!(game.get_user_role(self.connection_id), Role::MisterX) {
            return Err(ConnectionError::NotAllowedForUser);
        }

        Ok(())
    }

    async fn assert_in_game(&self) -> Result<(), ConnectionError> {
        if self.game_id().await.is_none() {
            return Err(ConnectionError::NotInGame);
        }

        Ok(())
    }

    async fn assert_in_lobby(&self) -> Result<(), ConnectionError> {
        if self.lobby_id().await.is_none() {
            return Err(ConnectionError::NotInLobby);
        }

        Ok(())
    }

    async fn handle_client_packet(&mut self, packet: ClientPacket) -> Result<(), ConnectionError> {
        println!("Received packet: {:?}", self.game_id().await);

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
                if self.lobby_id().await.is_some() {
                    return Err(ConnectionError::GameAlreadyJoined);
                }

                self.lobby_service.lock().await.join(
                    self.connection_id,
                    &packet.id,
                    self.sender().await,
                )?;

                self.set_lobby_id(Some(packet.id)).await;
            }
            ClientPacket::StartGame => {
                self.assert_in_lobby().await?;

                let lobby_id = self.lobby_id().await.unwrap();

                let mut ref_lobby_service = self.lobby_service.lock().await;
                let lobby = ref_lobby_service.get_lobby(&lobby_id)?;

                self.game_service
                    .lock()
                    .await
                    .add_game_from_lobby(lobby, &lobby_id)?;

                for player in &lobby.players {
                    let mut connections = self.ws_connection_service.lock().await;

                    let _ = connections.set_game_id(player.uuid, Some(lobby_id));
                    let _ = connections.set_lobby_id(player.uuid, None);
                }

                ref_lobby_service.close_lobby(&lobby_id);

                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.game_id().await.unwrap())?;
                game.start().await;
            }
            ClientPacket::MoveMisterX(packet) => {
                self.assert_in_game().await?;

                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.game_id().await.unwrap())?;

                self.assert_own_round(game)?;
                self.assert_detective(game)?;

                game.move_mister_x(
                    packet
                        .into_iter()
                        .map(|packet| (packet.station_id, packet.transport_type))
                        .collect(),
                )?;
            }
            ClientPacket::MoveDetective(packet) => {
                self.assert_in_game().await?;

                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(&self.game_id().await.unwrap())?;

                self.assert_own_round(game)?;
                self.assert_mister_x(game)?;

                game.move_detective(packet.color, packet.station_id, packet.transport_type)
                    .await?;
            }
            ClientPacket::SubmitMove => {
                self.assert_in_game().await?;

                let game_id = &self.game_id().await.unwrap();
                let mut ref_game_service = self.game_service.lock().await;
                let game = ref_game_service.get_game_mut(game_id)?;

                self.assert_own_round(game)?;

                let ended = game.end_move().await?;
                drop(ref_game_service);

                if ended {
                    self.game_service.lock().await.close_game(game_id).await;
                }
            }
        }

        Ok(())
    }
}
