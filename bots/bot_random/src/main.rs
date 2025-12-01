use game::{data::Connection, event::GameState};
use map_utils::{all_valid_detective_moves, all_valid_mister_x_moves};
use rand::seq::IndexedRandom;
use runtime::{DetectiveAction, MisterXAction};

pub struct Bot {
    connections: Vec<Connection>,
}

impl runtime::Bot for Bot {
    fn new(data: runtime::GameData) -> Self {
        Bot {
            connections: data.connections,
        }
    }

    fn next_mister_x_move(&mut self, game_state: &GameState) -> runtime::MisterXAction {
        let current_location = game_state.mister_x.station_id.unwrap();

        let valid_moves = all_valid_mister_x_moves(
            &self.connections,
            current_location,
            &game_state.mister_x.abilities,
        );

        let mut rand = rand::rng();
        let (station, action_type) = valid_moves.choose(&mut rand).unwrap().clone();

        MisterXAction {
            first_move: runtime::MisterXMove {
                station: station,
                action_type: action_type,
            },
            second_move: None,
        }
    }

    fn next_detective_move(&mut self, game_state: &GameState) -> runtime::DetectiveAction {
        let mut moves = vec![];

        for player in &game_state.players {
            let valid_moves = all_valid_detective_moves(
                &self.connections,
                player.station_id,
                &player.available_transport,
            );

            let mut rand = rand::rng();
            let (station, action_type) = valid_moves.choose(&mut rand).unwrap().clone();

            moves.push(runtime::DetectiveMove {
                color: player.color.to_string(),
                station,
                action_type,
            });
        }

        DetectiveAction {
            moves: moves.try_into().unwrap(),
        }
    }
}

fn main() {
    runtime::run_from_cli::<Bot>();
}
