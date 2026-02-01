use std::time::Duration;

use futures_timer::Delay;
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, canvas,
    div, fill, point, prelude::*, px, rgb, size,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .child(
                div()
                    .child(format!("Hello, {}!", &self.text))
                    .text_xl()
                    .text_color(rgb(0xffffff))
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                        println!("Mouse down event");
                    }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    |_, _, window, _| {
                        let rect = Bounds::new(point(px(50.), px(50.)), size(px(100.), px(100.)));
                        window.paint_quad(fill(rect, rgb(0xff00ff)));
                    },
                )
                .w_full()
                .h_full(),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();

        cx.background_spawn(async {
            loop {
                println!("Hello, World! - from background");
                Delay::new(Duration::from_secs(1)).await;
            }
        })
        .detach();

        cx.activate(true);
    })
}
