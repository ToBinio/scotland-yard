use gpui::{App, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::sidebar::{EventListener, button::Button};

#[derive(IntoElement, Default)]
pub struct Sidebar {
    on_start_game: Option<EventListener>,
}

impl Sidebar {
    pub fn on_start_game(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_start_game = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().w(px(200.)).h_full().child(Button::new(
            "start_game",
            "Start Game".to_string(),
            self.on_start_game.expect("No on_start_game set"),
        ))
    }
}
