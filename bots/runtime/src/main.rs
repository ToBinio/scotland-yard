use clap::{Parser, Subcommand};
use packets::{ClientPacket, ServerPacket};
use runtime::connection::Connection;

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
}

fn main() {
    let args = Args::parse();

    match args.sub {
        SubCommands::CreateGame { server } => {
            let mut connection = Connection::new(&server);

            connection.send(ClientPacket::CreateGame(packets::CreateGamePacket {
                number_of_detectives: 4,
            }));

            let msg = connection.receive();

            if let ServerPacket::Game(game) = msg {
                println!("created Game with id: {:?}", game.id);
            }
        }
    }
}
