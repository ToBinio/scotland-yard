use gpui::{App, ClickEvent, Context, Entity, Window, div, prelude::*, rgb};
use packets::{ClientPacket, ServerPacket};

use crate::{map::Map, sidebar::Sidebar, websocket::Connection};

pub struct Root {
    map: Entity<Map>,
    ws_connection: Connection,
}

impl Root {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let ws_connection = Connection::new("http://localhost:8081");

        Self {
            map: cx.new(Map::new),
            ws_connection,
        }
    }

    fn create_game(&mut self, _event: &ClickEvent, _window: &mut Window, _app: &mut Context<Self>) {
        self.ws_connection
            .send(ClientPacket::CreateGame(packets::CreateGamePacket {
                number_of_detectives: 4,
            }));

        let msg = self.ws_connection.receive();

        if let ServerPacket::Game(game) = msg {
            println!("{}", game.id)
        };
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x505050))
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .child(
                div()
                    .child("Scotland Yard")
                    .text_xl()
                    .text_color(rgb(0xffffff)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .size_full()
                    .child(Sidebar::new(cx.listener(Self::create_game)))
                    .child(self.map.clone()),
            )
    }
}
