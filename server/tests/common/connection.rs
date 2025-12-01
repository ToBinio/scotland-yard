use axum_test::{TestServer, TestWebSocket};
use serde::Deserialize;
use serde_json::json;

use crate::common::{
    data::Game,
    ws::{assert_receive_message, get_ws_connection, send_message},
};

pub async fn create_game(socket: &mut TestWebSocket) -> String {
    send_message(
        socket,
        "createGame",
        Some(json!({
            "number_of_detectives": 4,
        })),
    )
    .await;

    #[derive(Debug, Deserialize)]
    struct GameCreated {
        id: String,
    }

    let response = assert_receive_message::<GameCreated>(socket, "game").await;

    response.unwrap().id
}

pub struct GameConnection {
    pub mister_x: TestWebSocket,
    pub detective: TestWebSocket,
}

pub async fn start_game(server: &mut TestServer) -> GameConnection {
    let mut player_1 = get_ws_connection(server).await;
    let mut player_2 = get_ws_connection(server).await;

    let game_id = create_game(&mut player_1).await;

    send_message(&mut player_1, "joinGame", Some(json!({ "id": game_id }))).await;
    send_message(&mut player_2, "joinGame", Some(json!({ "id": game_id }))).await;

    send_message(&mut player_2, "startGame", None).await;

    #[derive(Debug, Deserialize)]
    struct GameStarted {
        role: String,
    }

    let response = assert_receive_message::<GameStarted>(&mut player_1, "gameStarted").await;
    let role_1 = response.unwrap().role;

    let _ = assert_receive_message::<GameStarted>(&mut player_2, "gameStarted").await;

    if role_1 == "detective" {
        GameConnection {
            mister_x: player_2,
            detective: player_1,
        }
    } else {
        GameConnection {
            mister_x: player_1,
            detective: player_2,
        }
    }
}

pub async fn start_game_with_colors(server: &mut TestServer) -> (GameConnection, Vec<String>) {
    let mut game = start_game(server).await;

    game.receive_start_move_message("mister_x").await;

    assert_receive_message::<Game>(&mut game.mister_x, "gameState").await;
    let game_state = assert_receive_message::<Game>(&mut game.detective, "gameState").await;
    let colors: Vec<_> = game_state
        .unwrap()
        .players
        .iter()
        .map(|player| player.color.clone())
        .collect();

    (game, colors)
}

impl GameConnection {
    pub async fn receive_start_move_message(&mut self, expected_role: &str) {
        Self::receive_start_move_message_for_player(&mut self.mister_x, expected_role).await;
        Self::receive_start_move_message_for_player(&mut self.detective, expected_role).await;
    }

    async fn receive_start_move_message_for_player(
        player: &mut TestWebSocket,
        expected_role: &str,
    ) {
        #[derive(Debug, Deserialize)]
        struct StartMove {
            role: String,
        }

        let message = assert_receive_message::<StartMove>(player, "startMove").await;
        assert_eq!(message.unwrap().role, expected_role);
    }

    pub async fn receive_game_ended_message(&mut self, expected_winner: &str) {
        #[derive(Debug, Deserialize)]
        struct GameEnded {
            winner: String,
        }

        let msg = assert_receive_message::<GameEnded>(&mut self.mister_x, "gameEnded")
            .await
            .unwrap();
        assert_eq!(msg.winner, expected_winner);
        assert_receive_message::<GameEnded>(&mut self.detective, "gameEnded")
            .await
            .unwrap();
        assert_eq!(msg.winner, expected_winner);

        assert_receive_message::<Game>(&mut self.mister_x, "gameState").await;
        let data = assert_receive_message::<Game>(&mut self.detective, "gameState")
            .await
            .unwrap();

        assert!(data.mister_x.station_id.is_some());
    }

    pub async fn send_detective_move(
        &mut self,
        color: &str,
        station: u8,
        transport_type: &str,
    ) -> Game {
        send_message(
            &mut self.detective,
            "moveDetective",
            Some(
                json!({ "color": color, "station_id": station, "transport_type": transport_type }),
            ),
        )
        .await;

        assert_receive_message::<Game>(&mut self.mister_x, "gameState").await;
        assert_receive_message::<Game>(&mut self.detective, "gameState")
            .await
            .unwrap()
    }

    pub async fn double_move(&mut self, colors: &[String]) {
        self.full_move_mister_x(110).await;

        self.full_move_detectives(
            &colors,
            &[106, 107, 108, 109],
            &["taxi", "bus", "bus", "taxi"],
        )
        .await;
        self.full_move_mister_x(104).await;

        self.full_move_detectives(
            &colors,
            &[100, 101, 102, 103],
            &["taxi", "bus", "bus", "taxi"],
        )
        .await;
    }

    pub async fn full_move_detectives(
        &mut self,
        colors: &[String],
        stations: &[u8; 4],
        transport: &[&str],
    ) -> Game {
        #[derive(Debug, Deserialize)]
        struct EndMove;

        let _ = self
            .send_detective_move(&colors[0], stations[0], transport[0])
            .await;
        let _ = self
            .send_detective_move(&colors[1], stations[1], transport[1])
            .await;
        let _ = self
            .send_detective_move(&colors[2], stations[2], transport[2])
            .await;
        let _ = self
            .send_detective_move(&colors[3], stations[3], transport[3])
            .await;

        send_message(&mut self.detective, "submitMove", None).await;

        assert_receive_message::<EndMove>(&mut self.mister_x, "endMove").await;
        assert_receive_message::<EndMove>(&mut self.detective, "endMove").await;

        self.receive_start_move_message("mister_x").await;

        assert_receive_message::<Game>(&mut self.mister_x, "gameState").await;
        assert_receive_message::<Game>(&mut self.detective, "gameState")
            .await
            .unwrap()
    }

    pub async fn full_move_mister_x(&mut self, station: u32) -> Game {
        #[derive(Debug, Deserialize)]
        struct EndMove;

        send_message(
            &mut self.mister_x,
            "moveMisterX",
            Some(json!([{ "station_id": station, "transport_type": "taxi" }])),
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
