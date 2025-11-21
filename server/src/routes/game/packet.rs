use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::services::lobby::LobbyId;

#[derive(Deserialize, Serialize)]
pub struct ErrorPacket {
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateGamePacket {
    pub number_of_detectives: u8,
}

#[derive(Deserialize, Serialize)]
pub struct GamePacket {
    pub id: LobbyId,
}

#[derive(Deserialize, Serialize)]
pub struct JoinGamePacket {
    pub id: LobbyId,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Detective,
    MisterX,
}

#[derive(Deserialize, Serialize)]
pub struct GameStartedPacket {
    pub role: Role,
}

pub enum ServerPacket {
    Error(ErrorPacket),
    Game(GamePacket),
    GameStarted(GameStartedPacket),
}

pub enum ClientPacket {
    CreateGame(CreateGamePacket),
    JoinGame(JoinGamePacket),
    StartGame,
}

#[derive(Error, Debug, PartialEq)]
pub enum PacketError {
    #[error("unknown packet")]
    UnknownPacket,
    #[error("invalid packet")]
    InvalidPacket,
}

impl Display for ServerPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, content) = match self {
            ServerPacket::Error(content) => {
                ("error", Some(serde_json::to_string(content).unwrap()))
            }
            ServerPacket::Game(content) => ("game", Some(serde_json::to_string(content).unwrap())),
            ServerPacket::GameStarted(content) => {
                ("gameStarted", Some(serde_json::to_string(content).unwrap()))
            }
        };

        if let Some(content) = content {
            f.write_fmt(format_args!("[{}] {}", name, content))
        } else {
            f.write_fmt(format_args!("[{}]", name))
        }
    }
}

impl ServerPacket {
    pub fn from_error(err: impl Error) -> ServerPacket {
        ServerPacket::Error(ErrorPacket {
            message: err.to_string(),
        })
    }
}

impl ClientPacket {
    pub fn from(message: &str) -> Result<ClientPacket, PacketError> {
        let mut split = message.splitn(2, " ");

        let name = split.next().unwrap().to_string();
        let content = split.next().map(|content| content.to_string());

        fn get_content<T: DeserializeOwned>(content: Option<String>) -> Result<T, PacketError> {
            serde_json::from_str(&content.ok_or(PacketError::InvalidPacket)?)
                .map_err(|_| PacketError::InvalidPacket)
        }

        match name.as_str() {
            "[createGame]" => Ok(ClientPacket::CreateGame(get_content(content)?)),
            "[joinGame]" => Ok(ClientPacket::JoinGame(get_content(content)?)),
            "[startGame]" => Ok(ClientPacket::StartGame),
            _ => Err(PacketError::UnknownPacket),
        }
    }
}
