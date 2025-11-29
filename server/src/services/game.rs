use std::{collections::HashMap, sync::Arc};

use rand::Rng;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    game::{
        Game, GameError,
        character::{detective::Detective, mister_x::MisterX},
    },
    services::{
        data::DataServiceHandle,
        lobby::{Lobby, LobbyId},
        ws_connection::WsConnectionServiceHandle,
    },
};

pub type GameId = Uuid;

pub type GameServiceHandle = Arc<Mutex<GameService>>;

#[derive(Error, Debug, PartialEq)]
pub enum GameServiceError {
    #[error(transparent)]
    Game(#[from] GameError),

    #[error("unknown game")]
    UnknownGame,
    #[error("game does not have enough players")]
    NotEnoughPlayers,
}

pub struct GameService {
    games: HashMap<GameId, Game>,
    data_service: DataServiceHandle,
    ws_connection_service: WsConnectionServiceHandle,
}

impl GameService {
    pub fn new(
        data_service: DataServiceHandle,
        ws_connection_service: WsConnectionServiceHandle,
    ) -> Self {
        Self {
            games: HashMap::new(),
            data_service,
            ws_connection_service,
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
            .map(|i| Detective::new(starting_stations[i], colors[i]))
            .collect();

        let detective_players = lobby
            .players
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != mister_x)
            .map(|(_, player)| player.clone())
            .collect();

        let game = Game::new(
            detective_players,
            detectives,
            lobby.players[mister_x].clone(),
            MisterX::new(*starting_stations.last().unwrap()),
            self.data_service.clone(),
        );

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

    pub async fn close_game(&mut self, game_id: &GameId) {
        let game = self.get_game(game_id).unwrap();

        for player in game.all_players() {
            let mut connections = self.ws_connection_service.lock().await;
            let _ = connections.set_game_id(player, None);
        }

        self.games.remove(game_id);
    }
}
