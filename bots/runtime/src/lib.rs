use std::{ops::Not, vec};

use clap::{Parser, command};
use game::{
    data::{Connection, Round, Station},
    event::{DetectiveActionType, GameState, MisterXActionType, Role},
};
use packets::{ClientPacket, JoinGamePacket, ServerPacket};

pub mod connection;

pub struct GameData {
    pub stations: Vec<Station>,
    pub connections: Vec<Connection>,
    pub rounds: Vec<Round>,
}

impl GameData {
    fn fetch(url: &str) -> Self {
        let stations = ureq::get(format!("{}/map/stations", url))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();

        let stations: Vec<Station> = serde_json::from_str(&stations).unwrap();

        let connections = ureq::get(format!("{}/map/connections", url))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();

        let connections: Vec<Connection> = serde_json::from_str(&connections).unwrap();

        let rounds = ureq::get(format!("{}/map/rounds", url))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();

        let rounds: Vec<Round> = serde_json::from_str(&rounds).unwrap();

        Self {
            stations,
            connections,
            rounds,
        }
    }
}

pub struct MisterXAction {
    pub first_move: MisterXMove,
    pub second_move: Option<MisterXMove>,
}

pub struct MisterXMove {
    pub station: u8,
    pub action_type: MisterXActionType,
}

#[derive(Debug)]
pub struct DetectiveAction {
    pub moves: [DetectiveMove; 4],
}

#[derive(Debug)]
pub struct DetectiveMove {
    pub color: String,
    pub station: u8,
    pub action_type: DetectiveActionType,
}

pub trait Bot {
    fn new(data: GameData) -> Self;
    fn next_mister_x_move(&mut self, game_state: &GameState) -> MisterXAction;
    fn next_detective_move(&mut self, game_state: &GameState) -> DetectiveAction;
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the server to connect to including http:// or https://
    #[arg(short, long)]
    server: String,

    /// UUID of the game to join
    #[arg(short, long)]
    game_id: String,
}

pub fn run_from_cli<B: Bot>() {
    let args = Args::parse();

    let mut bot = B::new(GameData::fetch(&args.server));

    let mut connection = connection::Connection::new(&args.server);

    let role = join_game(&mut connection, args.game_id);
    println!("game stared: playing as {:?}", role);

    let winner = play_game(&mut bot, &mut connection, &role);
    println!("game ended: winner is {:?}", winner);
}

fn join_game(connection: &mut connection::Connection, game_id: String) -> Role {
    connection.send(ClientPacket::JoinGame(JoinGamePacket {
        id: game_id.try_into().unwrap(),
    }));

    connection.send(ClientPacket::StartGame);

    loop {
        match connection.receive() {
            ServerPacket::GameStarted(packet) => return packet.role,
            _ => {}
        }
    }
}

fn play_game<B: Bot>(bot: &mut B, connection: &mut connection::Connection, role: &Role) -> Role {
    loop {
        match connection.receive() {
            ServerPacket::StartMove(packet) => {
                if packet.role.eq(role).not() {
                    continue;
                }

                let ServerPacket::GameState(state) = connection.receive() else {
                    panic!("Expected GameState packet");
                };

                match role {
                    Role::Detective => {
                        let action = bot.next_detective_move(&state);

                        for action in action.moves {
                            connection.send(ClientPacket::MoveDetective(
                                packets::MoveDetectivePacket {
                                    color: action.color,
                                    station_id: action.station,
                                    transport_type: action.action_type,
                                },
                            ));
                        }
                    }
                    Role::MisterX => {
                        let action = bot.next_mister_x_move(&state);

                        let mut moves = vec![packets::MoveMisterXPacket {
                            station_id: action.first_move.station,
                            transport_type: action.first_move.action_type,
                        }];

                        if let Some(second) = action.second_move {
                            moves.push(packets::MoveMisterXPacket {
                                station_id: second.station,
                                transport_type: second.action_type,
                            });
                        }

                        connection.send(ClientPacket::MoveMisterX(moves));
                    }
                }

                connection.send(ClientPacket::SubmitMove);
            }
            ServerPacket::GameEnded(packet) => {
                return packet.winner;
            }
            _ => {}
        }
    }
}
