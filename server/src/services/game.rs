use std::{collections::HashMap, sync::Arc};

use rand::Rng;
use thiserror::Error;
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

use crate::{
    routes::game::packet::{GameStartedPacket, Role, ServerPacket},
    services::lobby::{Lobby, LobbyId},
};

pub type GameId = Uuid;

pub type GameServiceHandle = Arc<Mutex<GameService>>;

struct Detective {
    color: String,
    station_id: u32,
    taxi: u32,
    bus: u32,
    underground: u32,
}

struct MisterX {
    station_id: u32,
    double_move: u32,
    hidden: u32,
}

struct Game {
    detective_ws: Vec<Sender<ServerPacket>>,
    detectives: Vec<Detective>,
    mister_x_ws: Sender<ServerPacket>,
    mister_x: MisterX,
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

        let detectives = (0..lobby.settings.number_of_detectives)
            .map(|_| Detective {
                color: "unset".to_string(),
                station_id: 0,
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
            detective_ws: detectives_ws,
            detectives,
            mister_x_ws: lobby.players[mister_x].ws_sender.clone(),
            mister_x: MisterX {
                station_id: 0,
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

        game.mister_x_ws
            .send(ServerPacket::GameStarted(GameStartedPacket {
                role: Role::MisterX,
            }))
            .await
            .unwrap();

        for player in &game.detective_ws {
            player
                .send(ServerPacket::GameStarted(GameStartedPacket {
                    role: Role::Detective,
                }))
                .await
                .unwrap();
        }

        Ok(())
    }
}
