use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    game::character::{
        Character,
        detective::{self, Detective},
        mister_x::{self, MisterX},
    },
    routes::game::packet::{
        DetectiveData, DetectiveTransportData, GameStartedPacket, GameStatePacket,
        MisterXAbilityData, MisterXData, ServerPacket, StartMovePacket,
    },
    services::lobby::{Player, PlayerId},
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
    ) -> Game {
        Game {
            active_role: Role::MisterX,
            game_round: 0,
            detective_players,
            detectives,
            mister_x_player,
            mister_x,
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

        self.send_game_state().await;
    }

    async fn send_game_state(&self) {
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
                station_id: None,
                abilities: MisterXAbilityData {
                    double_move: self.mister_x.double_moves(),
                    hidden: self.mister_x.hidden(),
                },
                moves: self.mister_x.action_types(),
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
                self.mister_x
                    .add_action(mister_x::Action::Single(mister_x::MoveData {
                        station: first.0,
                        action_type: first.1.clone(),
                    }));
            }
            (Some(first), Some(second)) => {
                if self.mister_x.double_moves() == 0 {
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
    ) {
        dbg!(&color);

        let detective = self
            .detectives
            .iter_mut()
            .find(|detective| detective.color() == color)
            .unwrap();

        detective.trim_actions(self.game_round as usize);

        detective.add_action(detective::Action {
            station: station_id,
            action_type: transport_type,
        });

        self.send_game_state().await;
    }

    pub async fn end_move(&mut self) -> Result<(), GameError> {
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
