use game::data::{Connection, Station, StationType};
use gpui::{IntoElement, PathBuilder, Rgba, Window, canvas};

use gpui::{
    App, Background, Bounds, PaintQuad, Pixels, Point, fill, point, prelude::*, px, rgb, size,
};
use itertools::Itertools;

use crate::map::RenderState;
use crate::map::data::MapData;

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
        for connection in self.map_data.connections() {
            self.draw_connection(window, connection, self.map_data.stations());
        }

        for station in self.map_data.stations() {
            self.draw_station(window, station);
        }
    }

    fn to_screen_space(&self, point: Point<Pixels>) -> Point<Pixels> {
        let point = point + self.render_state.offset;
        let point = point * self.render_state.zoom;
        point + self.center
    }

    fn draw_station(&self, window: &mut Window, station: &Station) {
        let position = self.to_screen_space(point(
            (station.pos_x as f64).into(),
            (station.pos_y as f64).into(),
        ));

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

    fn draw_connection(&self, window: &mut Window, connection: &Connection, stations: &[Station]) {
        let (width, color) = station_type_settings(&connection.mode);

        let from = stations.iter().find(|s| s.id == connection.from).unwrap();
        let to = stations.iter().find(|s| s.id == connection.to).unwrap();

        let mut builder = PathBuilder::stroke(width / 4.);
        builder.move_to(self.to_screen_space(point(px(from.pos_x as f32), px(from.pos_y as f32))));
        builder.line_to(self.to_screen_space(point(px(to.pos_x as f32), px(to.pos_y as f32))));
        let path = builder.build().unwrap();

        window.paint_path(path, color);
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
        game::data::StationType::Water => (px(20.0), rgb(0x0000FF)),
    }
}
