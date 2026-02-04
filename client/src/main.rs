use gpui::{
    App, Application, Background, Bounds, ContentMask, Context, Entity, PaintQuad, Pixels, Point,
    Window, WindowBounds, WindowOptions, canvas, div, fill, point, prelude::*, px, rgb, size,
};
use itertools::Itertools;

use crate::map_data::MapData;

mod map_data;

struct HelloWorld {
    map_data: Entity<MapData>,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let map_data = self.map_data.clone();

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
                    .child(format!(
                        "Hello, {}!",
                        map_data
                            .read(cx)
                            .stations()
                            .get(0)
                            .map(|station| station.id.to_string())
                            .unwrap_or("idk".to_string())
                    ))
                    .text_xl()
                    .text_color(rgb(0xffffff))
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                        println!("Mouse down event");
                    }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let stations = map_data.read(cx).stations();

                        window.with_content_mask(Some(ContentMask { bounds }), |window| {
                            for station in stations {
                                station
                                    .types
                                    .iter()
                                    .map(|station_type| match station_type {
                                        game::data::StationType::Taxi => (px(11.0), rgb(0xff00ff)),
                                        game::data::StationType::Bus => (px(8.0), rgb(0x00ff00)),
                                        game::data::StationType::Underground => {
                                            (px(5.0), rgb(0x0000ff))
                                        }
                                        game::data::StationType::Water => unreachable!(),
                                    })
                                    .sorted_by(|(size_a, _), (size_b, _)| {
                                        size_a.cmp(size_b).reverse()
                                    })
                                    .for_each(|(size, color)| {
                                        window.paint_quad(fill_circle(
                                            point(
                                                (station.pos_x as f64 / 2.0).into(),
                                                (station.pos_y as f64 / 2.0).into(),
                                            ),
                                            size / 2.0,
                                            color,
                                        ));
                                    });
                            }
                        });
                    },
                )
                .bg(rgb(0xffffff))
                .overflow_hidden()
                .w_full()
                .h_full(),
            )
    }
}

pub fn fill_circle(
    center: Point<Pixels>,
    radius: Pixels,
    background: impl Into<Background>,
) -> PaintQuad {
    let bounds = Bounds::new(
        center - point(radius, radius),
        size(radius * 2.0, radius * 2.0),
    );

    fill(bounds, background).corner_radii(radius)
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
