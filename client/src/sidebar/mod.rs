use gpui::{IntoElement, ParentElement, RenderOnce, Styled, div, px};

use crate::sidebar::button::Button;

mod button;

#[derive(IntoElement)]
pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().w(px(200.)).h_full().child(Button::new(
            "Button".to_string(),
            "Click me".to_string(),
            |_, _, _| println!("Button clicked"),
        ))
    }
}
