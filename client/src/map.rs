use game::data::{Station, StationType};
use gpui::{Entity, IntoElement, Rgba, Window, canvas, div};

use crate::map_data::MapData;
use gpui::{
    App, Background, Bounds, PaintQuad, Pixels, Point, fill, point, prelude::*, px, rgb, size,
};
use itertools::Itertools;

#[derive(IntoElement)]
pub struct Map {
    map_data: Entity<MapData>,
    render_state: RenderState,
}

struct RenderState {
    center: Point<Pixels>,
}

impl Map {
    pub fn new(data: Entity<MapData>) -> Self {
        Self {
            map_data: data,
            render_state: RenderState {
                center: point(px(0.0), px(0.0)),
            },
        }
    }

    fn draw(&self, window: &mut Window, cx: &mut App) {
        let stations = self.map_data.read(cx).stations();

        for station in stations {
            self.draw_station(window, station);
        }
    }

    fn draw_station(&self, window: &mut Window, station: &Station) {
        let position = point(
            (station.pos_x as f64 / 2.0).into(),
            (station.pos_y as f64 / 2.0).into(),
        ) + self.render_state.center;

        station
            .types
            .iter()
            .map(station_type_settings)
            .sorted_by(|(size_a, _), (size_b, _)| size_a.cmp(size_b).reverse())
            .for_each(|(size, color)| {
                window.paint_quad(fill_circle(position, size / 2.0, color));
            });
    }
}

impl RenderOnce for Map {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        self.render_state.center = bounds.center();

                        self.draw(window, cx)
                    },
                )
                .bg(rgb(0xffffff))
                .size_full(),
            )
            .overflow_hidden()
            .size_full()
    }
}

fn fill_circle(
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

fn station_type_settings(station_type: &StationType) -> (Pixels, Rgba) {
    match station_type {
        game::data::StationType::Taxi => (px(11.0), rgb(0xff00ff)),
        game::data::StationType::Bus => (px(8.0), rgb(0x00ff00)),
        game::data::StationType::Underground => (px(5.0), rgb(0x0000ff)),
        game::data::StationType::Water => unreachable!(),
    }
}
