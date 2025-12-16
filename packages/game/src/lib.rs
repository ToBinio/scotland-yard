use std::ops::Not;

use thiserror::Error;

use crate::{
    character::{
        ActionTypeTrait, Character,
        detective::{self, Detective},
        mister_x::{self, MisterX},
    },
    data::{Connection, Round},
    event::{
        DetectiveActionType, DetectiveData, DetectiveTransportData, EventListener, GameState,
        MisterXAbilityData, MisterXActionType, MisterXData, Role,
    },
    map_utils::all_valid_detective_moves,
    replay::Replay,
};

mod character;
pub mod data;
pub mod event;
pub mod map_utils;
pub mod replay;

#[derive(Error, Debug, PartialEq)]
pub enum GameError {
    #[error("invalid move")]
    InvalidMove,
    #[error("not all moved")]
    NotAllMoved,
}

pub struct Game<E: EventListener> {
    active_role: Role,
    game_round: u8,

    connections: Vec<Connection>,
    rounds: Vec<Round>,

    detectives: Vec<Detective>,
    mister_x: MisterX,

    event_listener: E,
}

impl<E: EventListener> Game<E> {
    pub fn new(
        detective_data: Vec<(String, u8)>,
        mister_x_start_station: u8,
        connections: Vec<Connection>,
        rounds: Vec<Round>,
        event_listener: E,
    ) -> Game<E> {
        let detectives = detective_data
            .into_iter()
            .map(|data| Detective::new(data.1, data.0))
            .collect();

        Game {
            active_role: Role::MisterX,
            game_round: 0,
            detectives,
            mister_x: MisterX::new(mister_x_start_station),
            event_listener,
            connections,
            rounds,
        }
    }

    pub fn active_role(&self) -> &Role {
        &self.active_role
    }

    pub fn event_listener(&self) -> &E {
        &self.event_listener
    }

    pub async fn start(&mut self) {
        self.event_listener.on_game_start().await;
        self.start_move(Role::MisterX).await;
    }

    pub async fn start_move(&mut self, role: Role) {
        self.active_role = role.clone();
        self.event_listener.on_start_round(&role).await;
        self.send_game_state(self.should_show_mister_x()).await;
    }

    async fn send_game_state(&self, show_mister_x: bool) {
        let state = GameState {
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
            round: self.game_round,
        };

        self.event_listener
            .on_game_state_update(state, show_mister_x)
            .await;
    }

    fn should_show_mister_x(&self) -> bool {
        match self.rounds.get(self.game_round as usize) {
            Some(round) => round.show_mister_x,
            None => false,
        }
    }

    pub fn move_mister_x(&mut self, moves: Vec<(u8, MisterXActionType)>) -> Result<(), GameError> {
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
        transport_type: DetectiveActionType,
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
                    if detective.actions().len() as u8 <= self.game_round
                        && all_valid_detective_moves(
                            &self.connections,
                            detective.station_id(),
                            &DetectiveTransportData {
                                taxi: detective.taxi(),
                                bus: detective.bus(),
                                underground: detective.underground(),
                            },
                        )
                        .is_empty()
                        .not()
                    {
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

        self.event_listener.on_end_move().await;

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
                if self.game_round as usize == self.rounds.len() {
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
        let max_actions = self
            .detectives
            .iter()
            .map(|detective| detective.actions().len())
            .max()
            .unwrap_or(0);
        let max_actions = max_actions.max(self.mister_x.actions().len());

        let mut actions = vec![];
        for i in 0..max_actions {
            if let Some(action) = self.mister_x.actions().get(i) {
                match action {
                    mister_x::Action::Single(action) => actions.push(replay::Action::MisterX {
                        station: action.station,
                        action_type: action.action_type.clone(),
                    }),
                    mister_x::Action::Double(action1, action2) => {
                        actions.push(replay::Action::MisterX {
                            station: action1.station,
                            action_type: action1.action_type.clone(),
                        });
                        actions.push(replay::Action::MisterX {
                            station: action2.station,
                            action_type: action2.action_type.clone(),
                        });
                    }
                }
            }

            for detective in &self.detectives {
                if let Some(action) = detective.actions().get(i) {
                    actions.push(replay::Action::Detective {
                        color: detective.color().to_string(),
                        action_type: action.action_type.clone(),
                        station: action.station,
                    });
                }
            }
        }

        let replay = Replay {
            mister_x_starting_station: self.mister_x.start_station(),
            detective_starting_stations: self
                .detectives
                .iter()
                .map(|d| (d.color().to_string(), d.start_station()))
                .collect(),
            actions,
            winner: winner.clone(),
        };

        self.event_listener.on_game_ended(&replay).await;
        self.send_game_state(true).await;
    }

    fn has_connection(&self, from: u8, to: u8, action_type: &dyn ActionTypeTrait) -> bool {
        self.connections
            .iter()
            .filter(|connection| {
                (connection.from == from && connection.to == to)
                    || (connection.from == to && connection.to == from)
            })
            .any(|connection| action_type.matches(&connection.mode))
    }
}
