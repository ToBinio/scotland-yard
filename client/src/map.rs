use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Point, Render, Styled, div,
    point, px,
};

use crate::{map_canvas::MapCanvas, map_data::MapData};

#[derive(Debug, Clone)]
pub struct RenderState {
    pub offset: Point<Pixels>,
}

pub struct Map {
    map_data: Entity<MapData>,
    render_state: Entity<RenderState>,
}

impl Map {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            map_data: cx.new(|cx| {
                let mut map_data = MapData::default();
                map_data.init(cx);
                map_data
            }),
            render_state: cx.new(|_| RenderState {
                offset: point(px(-750.0), px(-500.0)),
            }),
        }
    }
}

impl Render for Map {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let map_data = self.map_data.read(cx);
        let render_state = self.render_state.read(cx);

        div()
            .child(MapCanvas::new(map_data.clone(), render_state.clone()))
            .overflow_hidden()
            .size_full()
    }
}
