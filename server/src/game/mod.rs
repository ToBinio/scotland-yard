use std::ops::Not;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    game::character::{
        ActionTypeTrait, Character,
        detective::{self, Detective},
        mister_x::{self, MisterX},
    },
    routes::game::packet::{
        DetectiveData, DetectiveTransportData, GameEndedPacket, GameStartedPacket, GameStatePacket,
        MisterXAbilityData, MisterXData, ServerPacket, StartMovePacket,
    },
    services::{
        data::DataServiceHandle,
        lobby::{Player, PlayerId},
    },
};

pub mod character;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Detective,
    MisterX,
}

#[derive(Error, Debug, PartialEq)]
pub enum GameError {
    #[error("invalid move")]
    InvalidMove,
    #[error("not all moved")]
    NotAllMoved,
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
    pub fn new(
        detective_players: Vec<Player>,
        detectives: Vec<Detective>,
        mister_x_player: Player,
        mister_x: MisterX,
        data_service: DataServiceHandle,
    ) -> Game {
        Game {
            active_role: Role::MisterX,
            game_round: 0,
            detective_players,
            detectives,
            mister_x_player,
            mister_x,
            data_service,
        }
    }

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

        self.send_game_state(self.should_show_mister_x()).await;
    }

    async fn send_game_state(&self, show_mister_x: bool) {
        let mut packet = GameStatePacket {
            players: self
                .detectives
                .iter()
                .map(|data| DetectiveData {
                    color: data.color().to_string(),
                    station_id: data.station_id(),
                    available_transport: DetectiveTransportData {
                        taxi: data.taxi(),
                        bus: data.bus(),
                        underground: data.underground(),
                    },
                })
                .collect(),
            mister_x: MisterXData {
                station_id: Some(self.mister_x.station_id()),
                abilities: MisterXAbilityData {
                    double_move: self.mister_x.double_moves(),
                    hidden: self.mister_x.hidden(),
                },
                moves: self.mister_x.action_types(),
            },
        };

        self.mister_x_player
            .ws_sender
            .send(ServerPacket::GameState(packet.clone()))
            .await
            .unwrap();

        if show_mister_x.not() {
            packet.mister_x.station_id = None;
        }

        for player in &self.detective_players {
            player
                .ws_sender
                .send(ServerPacket::GameState(packet.clone()))
                .await
                .unwrap();
        }
    }

    fn should_show_mister_x(&self) -> bool {
        match self
            .data_service
            .get_all_rounds()
            .get(self.game_round as usize)
        {
            Some(round) => round.show_mister_x,
            None => false,
        }
    }

    pub fn all_players(&self) -> Vec<Uuid> {
        let mut players = vec![];
        players.extend(self.detective_players.iter().map(|player| player.uuid));
        players.push(self.mister_x_player.uuid);
        players
    }

    async fn send_all(&self, packet: ServerPacket) {
        for player in &self.detective_players {
            player.ws_sender.send(packet.clone()).await.unwrap();
        }
        self.mister_x_player.ws_sender.send(packet).await.unwrap();
    }

    pub fn move_mister_x(
        &mut self,
        moves: Vec<(u8, mister_x::ActionType)>,
    ) -> Result<(), GameError> {
        if moves.len() > 2 {
            return Err(GameError::InvalidMove);
        }

        self.mister_x.trim_actions(self.game_round as usize);

        let (first, second) = (moves.first(), moves.get(1));

        match (first, second) {
            (Some(first), None) => {
                if !self.mister_x.can_do_action(&first.1) {
                    return Err(GameError::InvalidMove);
                }

                if self
                    .has_connection(self.mister_x.station_id(), first.0, &first.1)
                    .not()
                {
                    return Err(GameError::InvalidMove);
                }

                self.mister_x
                    .add_action(mister_x::Action::Single(mister_x::MoveData {
                        station: first.0,
                        action_type: first.1.clone(),
                    }));
            }
            (Some(first), Some(second)) => {
                if !self.mister_x.can_do_action(&first.1) || !self.mister_x.can_do_action(&second.1)
                {
                    return Err(GameError::InvalidMove);
                }

                if self.mister_x.double_moves() == 0 {
                    return Err(GameError::InvalidMove);
                }

                if self
                    .has_connection(self.mister_x.station_id(), first.0, &first.1)
                    .not()
                    || self.has_connection(first.0, second.0, &second.1).not()
                {
                    return Err(GameError::InvalidMove);
                }

                self.mister_x.add_action(mister_x::Action::Double(
                    mister_x::MoveData {
                        station: first.0,
                        action_type: first.1.clone(),
                    },
                    mister_x::MoveData {
                        station: second.0,
                        action_type: second.1.clone(),
                    },
                ));
            }
            _ => return Err(GameError::InvalidMove),
        }

        Ok(())
    }

    pub async fn move_detective(
        &mut self,
        color: String,
        station_id: u8,
        transport_type: detective::ActionType,
    ) -> Result<(), GameError> {
        let detective = self
            .detectives
            .iter_mut()
            .find(|detective| detective.color() == color)
            .unwrap();

        detective.trim_actions(self.game_round as usize);

        if !detective.can_do_action(&transport_type) {
            return Err(GameError::InvalidMove);
        }

        let detective_station = detective.station_id();
        if self
            .has_connection(detective_station, station_id, &transport_type)
            .not()
        {
            return Err(GameError::InvalidMove);
        }

        let detective = self
            .detectives
            .iter_mut()
            .find(|detective| detective.color() == color)
            .unwrap();

        detective.add_action(detective::Action {
            station: station_id,
            action_type: transport_type,
        });

        self.send_game_state(self.should_show_mister_x()).await;

        Ok(())
    }

    /// returns true if the game is over, false otherwise
    pub async fn end_move(&mut self) -> Result<bool, GameError> {
        match self.active_role {
            Role::Detective => {
                for detective in &self.detectives {
                    if detective.actions().len() as u8 <= self.game_round {
                        return Err(GameError::NotAllMoved);
                    }
                }
            }
            Role::MisterX => {
                if self.mister_x.actions().len() as u8 <= self.game_round {
                    return Err(GameError::NotAllMoved);
                }
            }
        };

        self.send_all(ServerPacket::EndMove).await;

        if self
            .detectives
            .iter()
            .any(|detective| detective.station_id() == self.mister_x.station_id())
        {
            self.end_game(Role::Detective).await;
            return Ok(true);
        }

        match self.active_role {
            Role::Detective => {
                self.game_round += 1;
                if self.game_round as usize == self.data_service.get_all_rounds().len() {
                    self.end_game(Role::MisterX).await;
                    return Ok(true);
                }

                self.start_move(Role::MisterX).await
            }
            Role::MisterX => self.start_move(Role::Detective).await,
        };

        Ok(false)
    }

    pub async fn end_game(&mut self, winner: Role) {
        self.send_all(ServerPacket::GameEnded(GameEndedPacket { winner }))
            .await;

        self.send_game_state(true).await;
    }

    pub fn get_user_role(&self, id: PlayerId) -> Role {
        if self.mister_x_player.uuid == id {
            Role::MisterX
        } else {
            Role::Detective
        }
    }

    fn has_connection(&self, from: u8, to: u8, action_type: &dyn ActionTypeTrait) -> bool {
        let connections = self.data_service.get_all_connections();

        connections
            .iter()
            .filter(|connection| {
                (connection.from == from && connection.to == to)
                    || (connection.from == to && connection.to == from)
            })
            .any(|connection| action_type.matches(&connection.mode))
    }
}
