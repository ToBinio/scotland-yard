use std::process::Command;

use clap::{Parser, Subcommand};
use packets::{ClientPacket, ServerPacket};
use runtime::connection::Connection;
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
        } => {
            let Some(id) = game_id(&server) else {
                println!("failed to create game");
                return;
            };

            let mut bot_a = Command::new(bot_a)
                .args(["--server", &server])
                .args(["--game-id", &id.to_string()])
                .arg("--simple-output")
                .spawn()
                .expect("failed to spawn bot_a");

            let mut bot_b = Command::new(bot_b)
                .args(["--server", &server])
                .args(["--game-id", &id.to_string()])
                .arg("--simple-output")
                .spawn()
                .expect("failed to spawn bot_b");

            bot_a.wait().expect("failed to wait for bot_a");
            bot_b.wait().expect("failed to wait for bot_b");
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
