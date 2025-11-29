use serde::Deserialize;

use crate::common::{
    connection::start_game_with_colors,
    test_server,
    ws::{assert_receive_message, send_message},
};

mod common;

#[derive(Debug, Deserialize)]
struct EndMove;

#[tokio::test]
async fn can_lose() {
    let mut server = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    game.full_move_mister_x(110).await;
    game.full_move_detectives(&colors, &[106, 107, 108, 109])
        .await;

    game.full_move_mister_x(104).await;
    game.full_move_detectives(&colors, &[100, 101, 102, 103])
        .await;

    game.full_move_mister_x(110).await;
    game.full_move_detectives(&colors, &[106, 107, 108, 109])
        .await;

    game.full_move_mister_x(104).await;
    game.full_move_detectives(&colors, &[100, 101, 102, 103])
        .await;

    game.full_move_mister_x(110).await;

    // Last move
    game.send_detective_move(&colors[0], 106, "taxi").await;
    game.send_detective_move(&colors[1], 107, "bus").await;
    game.send_detective_move(&colors[2], 108, "bus").await;
    game.send_detective_move(&colors[3], 109, "taxi").await;

    send_message(&mut game.detective, "submitMove", None).await;

    assert_receive_message::<EndMove>(&mut game.mister_x, "endMove").await;
    assert_receive_message::<EndMove>(&mut game.detective, "endMove").await;

    game.receive_game_ended_message("mister_x").await;
}
