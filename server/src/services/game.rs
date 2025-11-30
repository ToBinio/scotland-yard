use std::{collections::HashMap, sync::Arc};

use game::{
    Game, GameError,
    event::{EventListener, GameState, Role},
};
use packets::{GameEndedPacket, GameStartedPacket, ServerPacket, StartMovePacket};
use rand::Rng;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::services::{
    data::DataServiceHandle,
    lobby::{Lobby, LobbyId, Player},
    ws_connection::WsConnectionServiceHandle,
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

pub struct GameEventListener {
    detective_players: Vec<Player>,
    mister_x_player: Player,
}

impl GameEventListener {
    pub fn all_players(&self) -> Vec<Uuid> {
        let mut players = vec![];
        players.extend(self.detective_players.iter().map(|player| player.uuid));
        players.push(self.mister_x_player.uuid);
        players
    }

    pub fn get_user_role(&self, id: Uuid) -> Role {
        if self.mister_x_player.uuid == id {
            Role::MisterX
        } else {
            Role::Detective
        }
    }

    async fn send_all(&self, packet: ServerPacket) {
        for player in &self.detective_players {
            player.ws_sender.send(packet.clone()).await.unwrap();
        }
        self.mister_x_player.ws_sender.send(packet).await.unwrap();
    }
}

impl EventListener for GameEventListener {
    async fn on_game_start(&self) {
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
    }

    async fn on_start_round(&self, role: &Role) {
        let packet = StartMovePacket { role: role.clone() };
        self.send_all(ServerPacket::StartMove(packet.clone())).await;
    }

    async fn on_end_move(&self) {
        self.send_all(ServerPacket::EndMove).await;
    }

    async fn on_game_ended(&self, role: &Role) {
        self.send_all(ServerPacket::GameEnded(GameEndedPacket {
            winner: role.clone(),
        }))
        .await;
    }

    async fn on_game_state_update(&self, mut state: GameState, show_mister_x: bool) {
        self.mister_x_player
            .ws_sender
            .send(ServerPacket::GameState(state.clone()))
            .await
            .unwrap();

        if !show_mister_x {
            state.mister_x.station_id = None;
        }

        for player in &self.detective_players {
            player
                .ws_sender
                .send(ServerPacket::GameState(state.clone()))
                .await
                .unwrap();
        }
    }
}

pub struct GameService {
    games: HashMap<GameId, Game<GameEventListener>>,
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

        let detective_players = lobby
            .players
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != mister_x)
            .map(|(_, player)| player.clone())
            .collect();

        let event_list = GameEventListener {
            detective_players,
            mister_x_player: lobby.players[mister_x].clone(),
        };

        let detectives_data = (0..lobby.settings.number_of_detectives)
            .map(|i| (colors[i].to_string(), starting_stations[i]))
            .collect();

        let game = Game::new(
            detectives_data,
            *starting_stations.last().unwrap(),
            self.data_service.get_all_connections(),
            self.data_service.get_all_rounds(),
            event_list,
        );

        self.games.insert(*lobby_id, game);

        Ok(())
    }

    pub async fn remove_game(&mut self, game_id: &GameId) {
        let game = self.get_game(game_id).unwrap();

        for player in game.event_listener().all_players() {
            let mut connections = self.ws_connection_service.lock().await;
            let _ = connections.set_game_id(player, None);
        }

        self.games.remove(game_id);
    }

    pub fn get_game(&self, game_id: &GameId) -> Result<&Game<GameEventListener>, GameServiceError> {
        self.games.get(game_id).ok_or(GameServiceError::UnknownGame)
    }

    pub fn get_game_mut(
        &mut self,
        game_id: &GameId,
    ) -> Result<&mut Game<GameEventListener>, GameServiceError> {
        self.games
            .get_mut(game_id)
            .ok_or(GameServiceError::UnknownGame)
    }
}
