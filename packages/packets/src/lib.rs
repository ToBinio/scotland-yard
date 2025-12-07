use std::{error::Error, fmt::Display};

use game::event::{DetectiveActionType, GameState, MisterXActionType, Role};
use packets_derive::Packets;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use uuid::Uuid;

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
    pub id: Uuid,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct JoinGamePacket {
    pub id: Uuid,
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
pub struct MoveMisterXPacket {
    pub station_id: u8,
    pub transport_type: MisterXActionType,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MoveDetectivePacket {
    pub color: String,
    pub station_id: u8,
    pub transport_type: DetectiveActionType,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GameEndedPacket {
    pub winner: Role,
}

#[derive(Error, Debug, PartialEq)]
pub enum PacketError {
    #[error("unknown packet")]
    UnknownPacket,
    #[error("invalid packet")]
    InvalidPacket,
}

#[derive(Packets, Clone)]
pub enum ServerPacket {
    Error(ErrorPacket),
    Game(GamePacket),
    GameStarted(GameStartedPacket),
    StartMove(StartMovePacket),
    GameState(GameState),
    EndMove,
    GameEnded(GameEndedPacket),
}

impl ServerPacket {
    pub fn from_error(err: impl Error) -> ServerPacket {
        ServerPacket::Error(ErrorPacket {
            message: err.to_string(),
        })
    }
}

#[derive(Packets, Clone)]
pub enum ClientPacket {
    CreateGame(CreateGamePacket),
    JoinGame(JoinGamePacket),
    StartGame,
    MoveMisterX(Vec<MoveMisterXPacket>),
    MoveDetective(MoveDetectivePacket),
    SubmitMove,
}
