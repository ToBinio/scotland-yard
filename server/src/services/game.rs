use std::{collections::HashMap, sync::Arc};

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    routes::game::packet::{
        DetectiveData, DetectiveTransportData, GameStartedPacket, GameStatePacket,
        MisterXAbilityData, MisterXData, MoveType, ServerPacket, StartMovePacket,
    },
    services::{
        data::DataServiceHandle,
        lobby::{Lobby, LobbyId, Player, PlayerId},
    },
};

pub type GameId = Uuid;

pub type GameServiceHandle = Arc<Mutex<GameService>>;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Detective,
    MisterX,
}

struct Move {
    station: u8,
    move_type: MoveType,
}

struct Detective {
    color: String,
    start_station_id: u8,
    moves: Vec<Move>,
}

impl Detective {
    pub fn station_id(&self) -> u8 {
        match self.moves.last() {
            Some(step) => step.station,
            None => self.start_station_id,
        }
    }

    pub fn taxi(&self) -> u8 {
        let count = self
            .moves
            .iter()
            .filter(|step| matches!(step.move_type, MoveType::Taxi))
            .count() as u8;

        10 - count
    }

    pub fn bus(&self) -> u8 {
        let count = self
            .moves
            .iter()
            .filter(|step| matches!(step.move_type, MoveType::Bus))
            .count() as u8;

        8 - count
    }

    pub fn underground(&self) -> u8 {
        let count = self
            .moves
            .iter()
            .filter(|step| matches!(step.move_type, MoveType::Underground))
            .count() as u8;

        4 - count
    }
}

enum MisterXMove {
    Single(Move),
    Double(Move, Move),
}

struct MisterX {
    start_station_id: u8,
    moves: Vec<MisterXMove>,
}

impl MisterX {
    pub fn station_id(&self) -> u8 {
        match self.moves.last() {
            Some(step) => match step {
                MisterXMove::Single(step) => step.station,
                MisterXMove::Double(_, step) => step.station,
            },
            None => self.start_station_id,
        }
    }

    /// Returns number of aviable hidden moves
    pub fn hidden(&self) -> u8 {
        let count = self
            .flat_moves()
            .into_iter()
            .filter(|step| step.eq(&MoveType::Hidden))
            .count() as u8;

        2 - count
    }

    /// Returns number of aviable double moves
    pub fn double_moves(&self) -> u8 {
        let count = self
            .moves
            .iter()
            .filter(|step| matches!(step, MisterXMove::Double(_, _)))
            .count() as u8;

        2 - count
    }

    pub fn flat_moves(&self) -> Vec<MoveType> {
        self.moves
            .iter()
            .flat_map(|step| match step {
                MisterXMove::Single(step) => vec![step.move_type.clone()],
                MisterXMove::Double(step1, step2) => {
                    vec![step1.move_type.clone(), step2.move_type.clone()]
                }
            })
            .collect()
    }
}

pub struct Game {
    active_role: Role,
    game_round: u8,
    data_service: DataServiceHandle,

    detective_players: Vec<Player>,
    detectives: Vec<Detective>,
    mister_x_player: Player,
    mister_x: MisterX,
}

impl Game {
    pub fn active_role(&self) -> &Role {
        &self.active_role
    }

    pub async fn start(&mut self) {
        self.mister_x_player
            .ws_sender
            .send(ServerPacket::GameStarted(GameStartedPacket {
                role: Role::MisterX,
            }))
            .await
            .unwrap();

        for player in &self.detective_players {
            player
                .ws_sender
                .send(ServerPacket::GameStarted(GameStartedPacket {
                    role: Role::Detective,
                }))
                .await
                .unwrap();
        }

        self.start_move(Role::MisterX).await;
    }

    pub async fn start_move(&mut self, role: Role) {
        self.active_role = role.clone();

        let packet = StartMovePacket { role };

        for player in &self.detective_players {
            player
                .ws_sender
                .send(ServerPacket::StartMove(packet.clone()))
                .await
                .unwrap();
        }

        self.mister_x_player
            .ws_sender
            .send(ServerPacket::StartMove(packet))
            .await
            .unwrap();

        self.send_game_state().await;
    }

    async fn send_game_state(&self) {
        let mut packet = GameStatePacket {
            players: self
                .detectives
                .iter()
                .map(|data| DetectiveData {
                    color: data.color.clone(),
                    station_id: data.station_id(),
                    available_transport: DetectiveTransportData {
                        taxi: data.taxi(),
                        bus: data.bus(),
                        underground: data.underground(),
                    },
                })
                .collect(),
            mister_x: MisterXData {
                station_id: None,
                abilities: MisterXAbilityData {
                    double_move: self.mister_x.double_moves(),
                    hidden: self.mister_x.hidden(),
                },
                moves: self.mister_x.flat_moves(),
            },
        };

        for player in &self.detective_players {
            player
                .ws_sender
                .send(ServerPacket::GameState(packet.clone()))
                .await
                .unwrap();
        }

        packet.mister_x.station_id = Some(self.mister_x.station_id());
        self.mister_x_player
            .ws_sender
            .send(ServerPacket::GameState(packet))
            .await
            .unwrap();
    }

    async fn send_all(&self, packet: ServerPacket) {
        for player in &self.detective_players {
            player.ws_sender.send(packet.clone()).await.unwrap();
        }
        self.mister_x_player.ws_sender.send(packet).await.unwrap();
    }

    pub fn move_mister_x(&mut self, moves: Vec<(u8, MoveType)>) -> Result<(), GameServiceError> {
        if moves.len() > 2 {
            return Err(GameServiceError::InvalidMove);
        }

        if self.mister_x.moves.len() as u8 > self.game_round {
            self.mister_x.moves.pop();
        }

        let (first, second) = (moves.first(), moves.get(1));

        match (first, second) {
            (Some(first), None) => {
                self.mister_x.moves.push(MisterXMove::Single(Move {
                    station: first.0,
                    move_type: first.1.clone(),
                }));
            }
            (Some(first), Some(second)) => {
                if self.mister_x.double_moves() == 0 {
                    return Err(GameServiceError::InvalidMove);
                }

                self.mister_x.moves.push(MisterXMove::Double(
                    Move {
                        station: first.0,
                        move_type: first.1.clone(),
                    },
                    Move {
                        station: second.0,
                        move_type: second.1.clone(),
                    },
                ));
            }
            _ => return Err(GameServiceError::InvalidMove),
        }

        Ok(())
    }

    pub async fn move_detective(
        &mut self,
        color: String,
        station_id: u8,
        transport_type: MoveType,
    ) {
        dbg!(&color);

        let detective = self
            .detectives
            .iter_mut()
            .find(|detective| detective.color == color)
            .unwrap();

        if detective.moves.len() as u8 > self.game_round {
            detective.moves.pop();
        }

        detective.moves.push(Move {
            station: station_id,
            move_type: transport_type,
        });

        self.send_game_state().await;
    }

    pub async fn end_move(&mut self) -> Result<(), GameServiceError> {
        match self.active_role {
            Role::Detective => {
                for detective in &self.detectives {
                    if detective.moves.len() as u8 <= self.game_round {
                        return Err(GameServiceError::NotAllMoved);
                    }
                }
            }
            Role::MisterX => {
                if self.mister_x.moves.len() as u8 <= self.game_round {
                    return Err(GameServiceError::NotAllMoved);
                }
            }
        };

        self.send_all(ServerPacket::EndMove).await;

        match self.active_role {
            Role::Detective => {
                self.game_round += 1;
                self.start_move(Role::MisterX).await
            }
            Role::MisterX => self.start_move(Role::Detective).await,
        };

        Ok(())
    }

    pub fn get_user_role(&self, id: PlayerId) -> Role {
        if self.mister_x_player.uuid == id {
            Role::MisterX
        } else {
            Role::Detective
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum GameServiceError {
    #[error("unknown game")]
    UnknownGame,
    #[error("game does not have enough players")]
    NotEnoughPlayers,
    #[error("invalid move")]
    InvalidMove,
    #[error("not all moved")]
    NotAllMoved,
}

pub struct GameService {
    games: HashMap<GameId, Game>,
    data_service: DataServiceHandle,
}

impl GameService {
    pub fn new(data_service: DataServiceHandle) -> Self {
        Self {
            games: HashMap::new(),
            data_service,
        }
    }

    pub fn add_game_from_lobby(
        &mut self,
        lobby: &Lobby,
        lobby_id: &LobbyId,
    ) -> Result<(), GameServiceError> {
        if lobby.players.len() < 2 {
            return Err(GameServiceError::NotEnoughPlayers);
        }

        let mut rng = rand::rng();

        let mister_x = rng.random_range(0..lobby.players.len());

        let colors = self.data_service.get_colors();
        let starting_stations = self
            .data_service
            .get_random_stations(lobby.settings.number_of_detectives + 1);

        let detectives = (0..lobby.settings.number_of_detectives)
            .map(|i| Detective {
                color: colors[i].to_string(),
                start_station_id: starting_stations[i],
                moves: vec![],
            })
            .collect();

        let detective_players = lobby
            .players
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != mister_x)
            .map(|(_, player)| player.clone())
            .collect();

        let game = Game {
            game_round: 0,
            active_role: Role::Detective,
            data_service: self.data_service.clone(),
            detective_players,
            detectives,
            mister_x_player: lobby.players[mister_x].clone(),
            mister_x: MisterX {
                start_station_id: *starting_stations.last().unwrap(),
                moves: Vec::new(),
            },
        };

        self.games.insert(*lobby_id, game);

        Ok(())
    }

    pub fn get_game(&self, game_id: &GameId) -> Result<&Game, GameServiceError> {
        self.games.get(game_id).ok_or(GameServiceError::UnknownGame)
    }

    pub fn get_game_mut(&mut self, game_id: &GameId) -> Result<&mut Game, GameServiceError> {
        self.games
            .get_mut(game_id)
            .ok_or(GameServiceError::UnknownGame)
    }
}
