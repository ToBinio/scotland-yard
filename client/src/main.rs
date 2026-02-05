use gpui::{
    App, Application, Bounds, Context, Entity, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

use crate::{map::Map, map_data::MapData};

pub mod map;
pub mod map_data;

struct HelloWorld {
    map_data: Entity<MapData>,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let map_data = self.map_data.clone();

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
            .child(Map::new(map_data.clone()))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        let map_data = MapData::new(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    map_data: map_data.clone(),
                })
            },
        )
        .unwrap();

        cx.activate(true);
    })
}
