use gpui::{Context, Entity, Window, div, prelude::*, rgb};

use crate::{map::Map, sidebar::Sidebar};

pub struct Root {
    map: Entity<Map>,
}

impl Root {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            map: cx.new(Map::new),
        }
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(Sidebar::new())
                    .child(self.map.clone()),
            )
    }
}
