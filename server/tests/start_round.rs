use serde::Deserialize;

use crate::common::{connection::start_game, data::Game, test_server, ws::assert_receive_message};

mod common;

#[tokio::test]
async fn correctly_starts_round() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    #[derive(Debug, Deserialize)]
    struct StartMove {
        role: String,
    }

    let message = assert_receive_message::<StartMove>(&mut game.mister_x, "startMove").await;
    assert_eq!(message.unwrap().role, "mister_x");
    let message = assert_receive_message::<StartMove>(&mut game.detective, "startMove").await;
    assert_eq!(message.unwrap().role, "mister_x");

    let message = assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    assert!(message.is_some());
    assert!(message.unwrap().mister_x.station_id.is_some());

    let message = assert_receive_message::<Game>(&mut game.detective, "gameState").await;
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
    assert_eq!(message.mister_x.moves.len(), 0);
}
