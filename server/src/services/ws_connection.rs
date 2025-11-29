use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

use crate::routes::game::packet::ServerPacket;

pub type WsConnectionServiceHandle = Arc<Mutex<WsConnectionService>>;

pub struct ConnectionData {
    lobby_id: Option<Uuid>,
    game_id: Option<Uuid>,
    ws_sender: Sender<ServerPacket>,
}

#[derive(Error, Debug, PartialEq)]
pub enum WsConnectionServiceError {
    #[error("unknown connection")]
    UnknownConnection,
}

#[derive(Default)]
pub struct WsConnectionService {
    connections: HashMap<Uuid, ConnectionData>,
}

impl WsConnectionService {
    pub fn add_connection(&mut self, connection_id: Uuid, ws_sender: Sender<ServerPacket>) {
        self.connections.insert(
            connection_id,
            ConnectionData {
                lobby_id: None,
                game_id: None,
                ws_sender,
            },
        );
    }
    pub fn ws_sender(
        &self,
        connection_id: Uuid,
    ) -> Result<Sender<ServerPacket>, WsConnectionServiceError> {
        self.connections
            .get(&connection_id)
            .ok_or(WsConnectionServiceError::UnknownConnection)
            .map(|data| data.ws_sender.clone())
    }

    pub fn lobby_id(&self, connection_id: Uuid) -> Result<Option<Uuid>, WsConnectionServiceError> {
        self.connections
            .get(&connection_id)
            .ok_or(WsConnectionServiceError::UnknownConnection)
            .map(|data| data.lobby_id)
    }

    pub fn set_lobby_id(
        &mut self,
        connection_id: Uuid,
        lobby_id: Option<Uuid>,
    ) -> Result<(), WsConnectionServiceError> {
        self.connections
            .get_mut(&connection_id)
            .ok_or(WsConnectionServiceError::UnknownConnection)
            .map(|data| data.lobby_id = lobby_id)
    }

    pub fn game_id(&self, connection_id: Uuid) -> Result<Option<Uuid>, WsConnectionServiceError> {
        self.connections
            .get(&connection_id)
            .ok_or(WsConnectionServiceError::UnknownConnection)
            .map(|data| data.game_id)
    }

    pub fn set_game_id(
        &mut self,
        connection_id: Uuid,
        game_id: Option<Uuid>,
    ) -> Result<(), WsConnectionServiceError> {
        self.connections
            .get_mut(&connection_id)
            .ok_or(WsConnectionServiceError::UnknownConnection)
            .map(|data| data.game_id = game_id)
    }
}
