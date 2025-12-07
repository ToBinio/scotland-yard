use std::{
    collections::HashMap,
    process::{Command, Stdio},
    thread,
};

use clap::{Parser, Subcommand};
use game::event::Role;
use packets::{ClientPacket, ServerPacket};
use runtime::{Output, connection::Connection};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    sub: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    CreateGame {
        #[arg(short, long)]
        server: String,
    },
    RunGame {
        #[arg(short, long)]
        server: String,

        #[arg(short, long, default_value_t = 1)]
        count: u8,

        #[arg(long)]
        bot_a: String,

        #[arg(long)]
        bot_b: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.sub {
        SubCommands::CreateGame { server } => {
            let id = game_id(&server);

            if let Some(id) = id {
                println!("created Game with id: {:?}", id);
            }
        }
        SubCommands::RunGame {
            server,
            bot_a,
            bot_b,
            count,
        } => {
            let winners = thread::scope(|s| {
                let handles: Vec<_> = (0..count)
                    .map(|_| s.spawn(|| run_game(&server, &bot_a, &bot_b)))
                    .collect();

                handles
                    .into_iter()
                    .map(|h| h.join().unwrap())
                    .collect::<Vec<_>>()
            });

            let mut counts: HashMap<Role, usize> = HashMap::new();
            for w in winners {
                *counts.entry(w).or_insert(0) += 1;
            }

            println!("{:#?}", counts);
        }
    }
}

fn game_id(server: &str) -> Option<Uuid> {
    let mut connection = Connection::new(server);

    connection.send(ClientPacket::CreateGame(packets::CreateGamePacket {
        number_of_detectives: 4,
    }));

    let msg = connection.receive();

    if let ServerPacket::Game(game) = msg {
        Some(game.id)
    } else {
        None
    }
}

fn run_game(server: &str, bot_a: &str, bot_b: &str) -> Role {
    let Some(id) = game_id(server) else {
        panic!("failed to create game");
    };

    let bot_a = Command::new(bot_a)
        .args(["--server", server])
        .args(["--game-id", &id.to_string()])
        .arg("--simple-output")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn bot_a");

    let bot_b = Command::new(bot_b)
        .args(["--server", server])
        .args(["--game-id", &id.to_string()])
        .arg("--simple-output")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn bot_b");

    let output_a = bot_a.wait_with_output().expect("failed to wait for bot_a");
    let output_b = bot_b.wait_with_output().expect("failed to wait for bot_b");

    let output_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();
    let output_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    let output_a = serde_json::from_str::<Output>(&output_a).unwrap();
    let output_b = serde_json::from_str::<Output>(&output_b).unwrap();

    if output_a.winner != output_b.winner {
        panic!("diffrent winners detective")
    }

    output_a.winner
}
