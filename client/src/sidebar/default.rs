use gpui::{App, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::sidebar::{EventListener, button::Button};

#[derive(IntoElement, Default)]
pub struct Sidebar {
    on_create_game: Option<EventListener>,
}

impl Sidebar {
    pub fn on_create_game(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_create_game = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().w(px(200.)).h_full().child(Button::new(
            "create_game",
            "Create Game".to_string(),
            self.on_create_game.expect("No on_create_game set"),
        ))
    }
}
