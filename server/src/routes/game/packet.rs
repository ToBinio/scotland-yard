use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::{
    game::{
        Role,
        character::{detective, mister_x},
    },
    services::lobby::LobbyId,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct ErrorPacket {
    pub message: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateGamePacket {
    pub number_of_detectives: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GamePacket {
    pub id: LobbyId,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct JoinGamePacket {
    pub id: LobbyId,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GameStartedPacket {
    pub role: Role,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StartMovePacket {
    pub role: Role,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DetectiveData {
    pub color: String,
    pub station_id: u8,
    pub available_transport: DetectiveTransportData,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DetectiveTransportData {
    pub taxi: u8,
    pub bus: u8,
    pub underground: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MisterXData {
    pub station_id: Option<u8>,
    pub abilities: MisterXAbilityData,
    pub moves: Vec<mister_x::ActionType>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MisterXAbilityData {
    pub double_move: u8,
    pub hidden: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GameStatePacket {
    pub players: Vec<DetectiveData>,
    pub mister_x: MisterXData,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MoveMisterXPacket {
    pub station_id: u8,
    pub transport_type: mister_x::ActionType,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MoveDetectivePacket {
    pub color: String,
    pub station_id: u8,
    pub transport_type: detective::ActionType,
}

#[derive(Clone)]
pub enum ServerPacket {
    Error(ErrorPacket),
    Game(GamePacket),
    GameStarted(GameStartedPacket),
    StartMove(StartMovePacket),
    GameState(GameStatePacket),
    EndMove,
}

pub enum ClientPacket {
    CreateGame(CreateGamePacket),
    JoinGame(JoinGamePacket),
    StartGame,
    MoveMisterX(Vec<MoveMisterXPacket>),
    MoveDetective(MoveDetectivePacket),
    SubmitMove,
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
            ServerPacket::StartMove(content) => {
                ("startMove", Some(serde_json::to_string(content).unwrap()))
            }
            ServerPacket::GameState(content) => {
                ("gameState", Some(serde_json::to_string(content).unwrap()))
            }
            ServerPacket::EndMove => ("endMove", None),
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
            "[moveMisterX]" => Ok(ClientPacket::MoveMisterX(get_content(content)?)),
            "[moveDetective]" => Ok(ClientPacket::MoveDetective(get_content(content)?)),
            "[submitMove]" => Ok(ClientPacket::SubmitMove),
            _ => Err(PacketError::UnknownPacket),
        }
    }
}
