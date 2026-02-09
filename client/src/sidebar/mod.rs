use gpui::{App, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::sidebar::button::Button;

mod button;

#[derive(IntoElement)]
pub struct Sidebar {
    on_create_game: Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
}

impl Sidebar {
    pub fn new(on_create_game: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        Self {
            on_create_game: Box::new(on_create_game),
        }
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().w(px(200.)).h_full().child(Button::new(
            "create_game".to_string(),
            "Create Game".to_string(),
            self.on_create_game,
        ))
    }
}
