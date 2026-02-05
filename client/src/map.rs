use gpui::{Entity, IntoElement, Window, canvas, div};

use crate::map_data::MapData;
use gpui::{
    App, Background, Bounds, PaintQuad, Pixels, Point, fill, point, prelude::*, px, rgb, size,
};
use itertools::Itertools;

#[derive(IntoElement)]
pub struct Map {
    map_data: Entity<MapData>,
}

impl Map {
    pub fn new(data: Entity<MapData>) -> Self {
        Self { map_data: data }
    }

    fn draw(&self, window: &mut Window, cx: &mut App) {
        let stations = self.map_data.read(cx).stations();

        for station in stations {
            station
                .types
                .iter()
                .map(|station_type| match station_type {
                    game::data::StationType::Taxi => (px(11.0), rgb(0xff00ff)),
                    game::data::StationType::Bus => (px(8.0), rgb(0x00ff00)),
                    game::data::StationType::Underground => (px(5.0), rgb(0x0000ff)),
                    game::data::StationType::Water => unreachable!(),
                })
                .sorted_by(|(size_a, _), (size_b, _)| size_a.cmp(size_b).reverse())
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
    }
}

impl RenderOnce for Map {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .child(
                canvas(
                    |_, _, _| (),
                    move |_bounds, _, window, cx| self.draw(window, cx),
                )
                .bg(rgb(0xffffff))
                .size_full(),
            )
            .overflow_hidden()
            .size_full()
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
