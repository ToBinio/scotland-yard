use game::{
    data::Connection,
    event::{DetectiveActionType, GameState, MisterXActionType},
};
use map_utils::all_taxi_connections;
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

        //todo: actually get all valid moves
        let valid_moves = all_taxi_connections(&self.connections, current_location);

        let mut rand = rand::rng();
        let station = valid_moves.choose(&mut rand).unwrap().clone();

        MisterXAction {
            first_move: runtime::MisterXMove {
                station: station,
                action_type: MisterXActionType::Taxi,
            },
            second_move: None,
        }
    }

    fn next_detective_move(&mut self, game_state: &GameState) -> runtime::DetectiveAction {
        let mut moves = vec![];

        for player in &game_state.players {
            //todo: actually get all valid moves
            let valid_moves = all_taxi_connections(&self.connections, player.station_id);

            let mut rand = rand::rng();
            let station = valid_moves.choose(&mut rand).unwrap().clone();

            moves.push(runtime::DetectiveMove {
                color: player.color.to_string(),
                station,
                action_type: DetectiveActionType::Taxi,
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
