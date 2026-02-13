use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::root::Root;

pub mod map;
pub mod root;
pub mod sidebar;
pub mod websocket;

fn main() {
    Application::new()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);

            let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let my_root = cx.new(|cx| Root::new(cx, window));
                    cx.new(|cx| gpui_component::Root::new(my_root, window, cx))
                },
            )
            .unwrap();

            cx.activate(true);
        })
}
