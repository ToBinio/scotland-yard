use serde::Deserialize;

use crate::common::{connection::start_game, test_server, ws::assert_receive_message};

mod common;

#[tokio::test]
async fn correctly_starts_round() {
    let mut server = test_server();
    let (mut mister_x, mut detective) = start_game(&mut server).await;

    #[derive(Debug, Deserialize)]
    struct StartMove {
        role: String,
    }

    let message = assert_receive_message::<StartMove>(&mut mister_x, "startMove").await;
    assert_eq!(message.unwrap().role, "mister_x");
    let message = assert_receive_message::<StartMove>(&mut detective, "startMove").await;
    assert_eq!(message.unwrap().role, "mister_x");

    #[derive(Debug, Deserialize)]
    struct Transport {
        taxi: u32,
        bus: u32,
        underground: u32,
    }

    #[derive(Debug, Deserialize)]
    struct PlayerGame {
        color: String,
        station_id: u32,
        available_transport: Transport,
    }

    #[derive(Debug, Deserialize)]
    struct Abilities {
        double_move: u32,
        hidden: u32,
    }

    #[derive(Debug, Deserialize)]
    struct MisterXGame {
        station_id: Option<u32>,
        abilities: Abilities,
    }

    #[derive(Debug, Deserialize)]
    struct Game {
        players: Vec<PlayerGame>,
        mister_x: MisterXGame,
    }

    let message = assert_receive_message::<Game>(&mut mister_x, "game").await;
    assert!(message.is_some());
    assert!(message.unwrap().mister_x.station_id.is_some());

    let message = assert_receive_message::<Game>(&mut detective, "game").await;
    assert!(message.is_some());

    let message = message.unwrap();
    assert!(message.mister_x.station_id.is_none());
    assert_eq!(message.players.len(), 4);

    assert_eq!(message.players[0].available_transport.taxi, 10);
    assert_eq!(message.players[0].available_transport.bus, 8);
    assert_eq!(message.players[0].available_transport.underground, 4);

    assert!(
        message.players[0].color != message.players[1].color
            && message.players[0].color != message.players[2].color
            && message.players[0].color != message.players[3].color
    );

    assert!(
        message.players[0].station_id != message.players[1].station_id
            && message.players[0].station_id != message.players[2].station_id
            && message.players[0].station_id != message.players[3].station_id
    );

    assert_eq!(message.mister_x.abilities.double_move, 2);
    assert_eq!(message.mister_x.abilities.hidden, 2);
}
