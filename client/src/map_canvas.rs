use game::data::{Station, StationType};
use gpui::{IntoElement, Rgba, Window, canvas};

use crate::{map::RenderState, map_data::MapData};
use gpui::{
    App, Background, Bounds, PaintQuad, Pixels, Point, fill, point, prelude::*, px, rgb, size,
};
use itertools::Itertools;

#[derive(IntoElement)]
pub struct MapCanvas {
    map_data: MapData,
    render_state: RenderState,

    center: Point<Pixels>,
}

impl MapCanvas {
    pub fn new(data: MapData, render_state: RenderState) -> Self {
        Self {
            map_data: data,
            render_state,
            center: Point::default(),
        }
    }

    fn draw(&self, window: &mut Window, _cx: &mut App) {
        let stations = self.map_data.stations();

        for station in stations {
            self.draw_station(window, station);
        }
    }

    fn draw_station(&self, window: &mut Window, station: &Station) {
        let position = point((station.pos_x as f64).into(), (station.pos_y as f64).into());
        let position = position + self.render_state.offset;
        let position = position * self.render_state.zoom;
        let position = position + self.center;

        station
            .types
            .iter()
            .map(station_type_settings)
            .sorted_by(|(size_a, _), (size_b, _)| size_a.cmp(size_b).reverse())
            .for_each(|(size, color)| {
                let size = (size / 2.0) * self.render_state.zoom;
                window.paint_quad(fill_circle(position, size, color));
            });
    }
}

impl RenderOnce for MapCanvas {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        canvas(
            |_, _, _| (),
            move |_bounds, _, window, cx| {
                self.center = window.bounds().center();
                self.draw(window, cx);
            },
        )
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
        game::data::StationType::Taxi => (px(8.0), rgb(0xF2C94C)),
        game::data::StationType::Bus => (px(12.0), rgb(0x27AE60)),
        game::data::StationType::Underground => (px(16.0), rgb(0x2F80ED)),
        game::data::StationType::Water => unreachable!(),
    }
}
