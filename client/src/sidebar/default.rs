use gpui::{
    App, ClickEvent, Context, Entity, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::*, px, rgb,
};
use gpui_component::input::{Input, InputState};

use crate::sidebar::{EventListener, button::Button};

pub struct SidebarState {
    input: Entity<InputState>,
}

impl SidebarState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| InputState::new(window, cx)),
        }
    }
}

#[derive(IntoElement)]
pub struct Sidebar {
    state: Entity<SidebarState>,
    on_create_game: Option<EventListener>,
}

impl Sidebar {
    pub fn new(state: &Entity<SidebarState>) -> Self {
        Self {
            on_create_game: None,
            state: state.clone(),
        }
    }

    pub fn on_create_game(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_create_game = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let state = self.state.read(cx);

        div()
            .w(px(200.))
            .h_full()
            .flex()
            .flex_col()
            .gap_1()
            .child(Button::new(
                "create_game",
                "Create Game".to_string(),
                self.on_create_game.expect("No on_create_game set"),
            ))
            .child(
                div()
                    .child(
                        Input::new(&state.input)
                            .bg(rgb(0x707070))
                            .text_color(rgb(0xffffff))
                            .rounded_md(),
                    )
                    .mx_2(),
            )
    }
}
