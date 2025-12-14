use serde::Deserialize;
use serde_json::json;

use crate::common::{
    connection::{GameConnection, start_game_with_colors},
    data::Game,
    test_server,
    ws::{assert_receive_error, assert_receive_message, send_message},
};

mod common;

#[tokio::test]
async fn mister_x_hidden() {
    let (mut server, _dir) = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    for _ in 0..2 {
        game.hidden_move_mister_x(110).await;

        game.full_move_detectives(
            &colors,
            &[106, 107, 108, 109],
            &["taxi", "bus", "bus", "taxi"],
        )
        .await;
        game.hidden_move_mister_x(104).await;

        game.full_move_detectives(
            &colors,
            &[100, 101, 102, 103],
            &["taxi", "bus", "bus", "taxi"],
        )
        .await;
    }

    game.hidden_move_mister_x(110).await;

    game.full_move_detectives(
        &colors,
        &[106, 107, 108, 109],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "hidden" }])),
    )
    .await;
    assert_receive_error(&mut game.mister_x, "invalid move").await;
}

impl GameConnection {
    pub async fn hidden_move_mister_x(&mut self, station: u32) -> Game {
        #[derive(Debug, Deserialize)]
        struct EndMove;

        send_message(
            &mut self.mister_x,
            "moveMisterX",
            Some(json!([{ "station_id": station, "transport_type": "hidden" }])),
        )
        .await;

        send_message(&mut self.mister_x, "submitMove", None).await;

        assert_receive_message::<EndMove>(&mut self.mister_x, "endMove").await;
        assert_receive_message::<EndMove>(&mut self.detective, "endMove").await;

        self.receive_start_move_message("detective").await;

        assert_receive_message::<Game>(&mut self.mister_x, "gameState").await;
        assert_receive_message::<Game>(&mut self.detective, "gameState")
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn mister_x_double() {
    let (mut server, _dir) = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    game.double_move_mister_x().await;
    game.full_move_detectives(
        &colors,
        &[106, 107, 108, 109],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;
    game.double_move_mister_x().await;

    game.full_move_detectives(
        &colors,
        &[100, 101, 102, 103],
        &["taxi", "bus", "bus", "taxi"],
    )
    .await;

    send_message(
        &mut game.mister_x,
        "moveMisterX",
        Some(json!([{ "station_id": 110, "transport_type": "taxi" },{ "station_id": 104, "transport_type": "taxi" }])),
    )
    .await;
    assert_receive_error(&mut game.mister_x, "invalid move").await;
}

impl GameConnection {
    pub async fn double_move_mister_x(&mut self) -> Game {
        #[derive(Debug, Deserialize)]
        struct EndMove;

        send_message(
            &mut self.mister_x,
            "moveMisterX",
            Some(json!([{ "station_id": 110, "transport_type": "taxi" },{ "station_id": 104, "transport_type": "taxi" }])),
        )
        .await;

        send_message(&mut self.mister_x, "submitMove", None).await;

        assert_receive_message::<EndMove>(&mut self.mister_x, "endMove").await;
        assert_receive_message::<EndMove>(&mut self.detective, "endMove").await;

        self.receive_start_move_message("detective").await;

        assert_receive_message::<Game>(&mut self.mister_x, "gameState").await;
        assert_receive_message::<Game>(&mut self.detective, "gameState")
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn detective_undeground() {
    let (mut server, _dir) = test_server();
    let (mut game, colors) = start_game_with_colors(&mut server).await;

    for _ in 0..2 {
        game.full_move_mister_x(110).await;

        game.full_move_detectives(
            &colors,
            &[106, 107, 108, 109],
            &["taxi", "bus", "bus", "underground"],
        )
        .await;
        game.full_move_mister_x(104).await;

        game.full_move_detectives(
            &colors,
            &[100, 101, 102, 103],
            &["taxi", "bus", "bus", "underground"],
        )
        .await;
    }

    game.full_move_mister_x(110).await;

    send_message(
        &mut game.detective,
        "moveDetective",
        Some(json!({ "color": &colors[3], "station_id": 109, "transport_type": "underground" })),
    )
    .await;

    assert_receive_error(&mut game.detective, "invalid move").await;
}
