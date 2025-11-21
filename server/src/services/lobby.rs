use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

use crate::routes::game::packet::ServerPacket;

pub struct Settings {
    pub number_of_detectives: u8,
}

pub struct Player {
    pub uuid: Uuid,
    pub ws_sender: Sender<ServerPacket>,
}

pub struct Lobby {
    pub settings: Settings,
    pub players: Vec<Player>,
}

pub type LobbyId = Uuid;
pub type PlayerId = Uuid;

pub type LobbyServiceHandle = Arc<Mutex<LobbyService>>;

#[derive(Error, Debug, PartialEq)]
pub enum LobbyServiceError {
    #[error("unknown lobby")]
    UnknownLobby,
}

#[derive(Default)]
pub struct LobbyService {
    lobbies: HashMap<LobbyId, Lobby>,
}

impl LobbyService {
    pub fn create(&mut self, number_of_detectives: u8) -> LobbyId {
        let id = Uuid::new_v4();

        self.lobbies.insert(
            id,
            Lobby {
                settings: Settings {
                    number_of_detectives,
                },
                players: vec![],
            },
        );

        id
    }

    pub fn get_lobby(&self, lobby_id: &LobbyId) -> Result<&Lobby, LobbyServiceError> {
        self.lobbies
            .get(lobby_id)
            .ok_or(LobbyServiceError::UnknownLobby)
    }

    pub fn join(
        &mut self,
        lobby_id: &LobbyId,
        sender: Sender<ServerPacket>,
    ) -> Result<PlayerId, LobbyServiceError> {
        let id = Uuid::new_v4();

        let lobby = self
            .lobbies
            .get_mut(lobby_id)
            .ok_or(LobbyServiceError::UnknownLobby)?;
        lobby.players.push(Player {
            uuid: id,
            ws_sender: sender,
        });

        Ok(id)
    }

    pub fn close_lobby(&mut self, lobby_id: &LobbyId) {
        self.lobbies.remove(lobby_id);
    }
}
