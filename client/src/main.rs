use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::root::Root;

pub mod map;
pub mod root;
pub mod sidebar;
pub mod websocket;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(Root::new),
        )
        .unwrap();

        cx.activate(true);
    })
}
