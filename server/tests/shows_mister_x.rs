use crate::common::{connection::start_game_with_colors, test_server};

mod common;

#[tokio::test]
async fn shows_mister_x() {
    let (mut server, _dir) = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    let state = game.full_move_mister_x(110).await;
    assert!(state.mister_x.station_id.is_none());

    game.full_move_detectives(
        &colors,
        &[106, 107, 108, 109],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;
    let state = game.full_move_mister_x(104).await;
    assert!(state.mister_x.station_id.is_none());

    game.full_move_detectives(
        &colors,
        &[100, 101, 102, 103],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;
    let state = game.full_move_mister_x(110).await;
    assert!(state.mister_x.station_id.is_some());

    game.full_move_detectives(
        &colors,
        &[106, 107, 108, 109],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;
    let state = game.full_move_mister_x(104).await;
    assert!(state.mister_x.station_id.is_none());
}
