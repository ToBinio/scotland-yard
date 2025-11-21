use std::{collections::HashMap, sync::Arc};

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

use crate::{
    routes::game::packet::{
        DetectiveData, DetectiveTransportData, GameStartedPacket, GameStatePacket,
        MisterXAbilityData, MisterXData, ServerPacket, StartMovePacket,
    },
    services::{
        data::DataService,
        lobby::{Lobby, LobbyId},
    },
};

pub type GameId = Uuid;

pub type GameServiceHandle = Arc<Mutex<GameService>>;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Detective,
    MisterX,
}

struct Detective {
    color: String,
    station_id: u8,
    taxi: u32,
    bus: u32,
    underground: u32,
}

struct MisterX {
    station_id: u8,
    double_move: u32,
    hidden: u32,
}

struct Game {
    data_service: Arc<DataService>,

    detective_ws: Vec<Sender<ServerPacket>>,
    detectives: Vec<Detective>,
    mister_x_ws: Sender<ServerPacket>,
    mister_x: MisterX,
}

impl Game {
    pub async fn start(&self) {
        self.mister_x_ws
            .send(ServerPacket::GameStarted(GameStartedPacket {
                role: Role::MisterX,
            }))
            .await
            .unwrap();

        for player in &self.detective_ws {
            player
                .send(ServerPacket::GameStarted(GameStartedPacket {
                    role: Role::Detective,
                }))
                .await
                .unwrap();
        }

        self.start_move(Role::MisterX).await;
    }

    pub async fn start_move(&self, role: Role) {
        let packet = StartMovePacket { role };

        for player in &self.detective_ws {
            player
                .send(ServerPacket::StartMove(packet.clone()))
                .await
                .unwrap();
        }

        self.mister_x_ws
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
                    station_id: data.station_id,
                    available_transport: DetectiveTransportData {
                        taxi: data.taxi,
                        bus: data.bus,
                        underground: data.underground,
                    },
                })
                .collect(),
            mister_x: MisterXData {
                station_id: None,
                abilities: MisterXAbilityData {
                    double_move: self.mister_x.double_move,
                    hidden: self.mister_x.hidden,
                },
            },
        };

        for player in &self.detective_ws {
            player
                .send(ServerPacket::GameState(packet.clone()))
                .await
                .unwrap();
        }

        packet.mister_x.station_id = Some(self.mister_x.station_id);
        self.mister_x_ws
            .send(ServerPacket::GameState(packet))
            .await
            .unwrap();
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum GameServiceError {
    #[error("unknown game")]
    UnknownGame,
    #[error("game does not have enough players")]
    NotEnoughPlayers,
}

#[derive(Default)]
pub struct GameService {
    games: HashMap<GameId, Game>,
    data_service: Arc<DataService>,
}

impl GameService {
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
                station_id: starting_stations[i],
                taxi: 10,
                bus: 8,
                underground: 4,
            })
            .collect();

        let detectives_ws = lobby
            .players
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != mister_x)
            .map(|(_, player)| player.ws_sender.clone())
            .collect();

        let game = Game {
            data_service: self.data_service.clone(),
            detective_ws: detectives_ws,
            detectives,
            mister_x_ws: lobby.players[mister_x].ws_sender.clone(),
            mister_x: MisterX {
                station_id: *starting_stations.last().unwrap(),
                double_move: 2,
                hidden: 2,
            },
        };

        self.games.insert(*lobby_id, game);

        Ok(())
    }

    fn get_game(&self, game_id: &GameId) -> Result<&Game, GameServiceError> {
        self.games.get(game_id).ok_or(GameServiceError::UnknownGame)
    }

    pub async fn start(&self, game_id: &GameId) -> Result<(), GameServiceError> {
        let game = self.get_game(game_id)?;

        game.start().await;

        Ok(())
    }
}
