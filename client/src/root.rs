use gpui::{ClickEvent, Context, Entity, Window, div, prelude::*, rgb};
use gpui_component::input::{Input, InputState};
use packets::{ClientPacket, ServerPacket};
use uuid::Uuid;

use crate::{
    map::Map,
    sidebar::{SidebarState, default, lobby},
    websocket::Connection,
};

pub struct Root {
    map: Entity<Map>,
    input: Entity<InputState>,
    ws_connection: Connection,
    game_state: GameState,
}

#[derive(Default)]
struct GameState {
    state: SidebarState,
    game_id: Option<Uuid>,
}

impl Root {
    pub fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        let ws_connection = Connection::new("http://localhost:8081");

        Self {
            map: cx.new(Map::new),
            input: cx.new(|cx| InputState::new(window, cx)),
            ws_connection,
            game_state: GameState::default(),
        }
    }

    fn create_game(&mut self, _event: &ClickEvent, _window: &mut Window, _app: &mut Context<Self>) {
        self.ws_connection
            .send(ClientPacket::CreateGame(packets::CreateGamePacket {
                number_of_detectives: 4,
            }));

        let msg = self.ws_connection.receive();

        if let ServerPacket::Game(game) = msg {
            println!("Created Game with id: {}", game.id);
            self.game_state.game_id = Some(game.id);

            self.connect_to_game();
        };
    }

    fn connect_to_game(&mut self) {
        let Some(id) = self.game_state.game_id else {
            println!("Tried to join without Id");
            return;
        };

        self.ws_connection
            .send(ClientPacket::JoinGame(packets::JoinGamePacket { id }));
        self.game_state.state = SidebarState::LOBBY;
    }

    fn start_game(&mut self, _event: &ClickEvent, _window: &mut Window, _app: &mut Context<Self>) {
        println!("Starting game");
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut inner = div().flex().flex_row().size_full();

        match &self.game_state.state {
            SidebarState::NONE => {
                inner = inner.child(
                    default::Sidebar::default().on_create_game(cx.listener(Self::create_game)),
                );
            }
            SidebarState::LOBBY => {
                inner = inner
                    .child(lobby::Sidebar::default().on_start_game(cx.listener(Self::start_game)));
            }
            SidebarState::GAME(_) => todo!(),
        }

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
            .child(inner.child(self.map.clone()))
            .child(Input::new(&self.input))
    }
}
