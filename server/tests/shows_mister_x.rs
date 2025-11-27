use crate::common::{connection::start_game, data::Game, test_server, ws::assert_receive_message};

mod common;

#[tokio::test]
async fn shows_mister_x() {
    let mut server = test_server();
    let mut game = start_game(&mut server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;
    let colors: Vec<_> = game_state
        .unwrap()
        .players
        .iter()
        .map(|player| player.color.clone())
        .collect();

    let state = game.full_move_mister_x(110).await;
    assert!(state.mister_x.station_id.is_none());

    game.full_move_detectives(&colors, &[106, 107, 108, 109])
        .await;
    let state = game.full_move_mister_x(104).await;
    assert!(state.mister_x.station_id.is_none());

    game.full_move_detectives(&colors, &[100, 101, 102, 103])
        .await;
    let state = game.full_move_mister_x(110).await;
    assert!(state.mister_x.station_id.is_some());

    game.full_move_detectives(&colors, &[106, 107, 108, 109])
        .await;
    let state = game.full_move_mister_x(104).await;
    assert!(state.mister_x.station_id.is_none());
}
