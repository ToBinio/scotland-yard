use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

use crate::routes::game::packet::{GameStartedPacket, Role, ServerPacket};

struct Settings {
    number_of_detectives: u8,
}

struct Player {
    uuid: Uuid,
    ws_sender: Sender<ServerPacket>,
}

struct Lobby {
    settings: Settings,
    players: Vec<Player>,
}

pub type LobbyId = Uuid;
pub type PlayerId = Uuid;

pub type LobbyServiceHandle = Arc<Mutex<LobbyService>>;

#[derive(Error, Debug, PartialEq)]
pub enum LobbyServiceError {
    #[error("unknown Lobby")]
    UnknownLobby,
    #[error("unknown Lobby")]
    NotEnoughPlayers,
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

    pub fn join(
        &mut self,
        lobby_id: LobbyId,
        sender: Sender<ServerPacket>,
    ) -> Result<PlayerId, LobbyServiceError> {
        let id = Uuid::new_v4();

        let lobby = self
            .lobbies
            .get_mut(&lobby_id)
            .ok_or(LobbyServiceError::UnknownLobby)?;
        lobby.players.push(Player {
            uuid: id,
            ws_sender: sender,
        });

        Ok(id)
    }

    pub async fn start(&self, lobby_id: LobbyId) -> Result<(), LobbyServiceError> {
        let lobby = self.lobbies.get(&lobby_id).unwrap();

        if lobby.players.len() < 2 {
            return Err(LobbyServiceError::NotEnoughPlayers);
        }

        lobby.players[0]
            .ws_sender
            .send(ServerPacket::GameStarted(GameStartedPacket {
                role: Role::MisterX,
            }))
            .await
            .unwrap();

        for player in &lobby.players[1..] {
            player
                .ws_sender
                .send(ServerPacket::GameStarted(GameStartedPacket {
                    role: Role::Detective,
                }))
                .await
                .unwrap();
        }

        Ok(())
    }
}
