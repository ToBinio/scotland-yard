use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, Pixels, Point, Render, ScrollWheelEvent, Styled, Window, div,
    point, px, rgb,
};

use crate::map::{canvas::MapCanvas, data::MapData};

mod canvas;
mod data;

#[derive(Debug, Clone)]
pub struct RenderState {
    pub offset: Point<Pixels>,
    pub zoom: f32,
}

pub struct Map {
    map_data: Entity<MapData>,
    render_state: Entity<RenderState>,

    is_dragging: bool,
    last_mouse_position: Point<Pixels>,
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
                zoom: 1.0,
            }),
            is_dragging: false,
            last_mouse_position: point(px(0.0), px(0.0)),
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_dragging = true;
        self.last_mouse_position = event.position;
        cx.notify();
    }

    fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.is_dragging = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_dragging {
            let delta = event.position - self.last_mouse_position;
            self.last_mouse_position = event.position;
            self.render_state.update(cx, |state, _| {
                state.offset += delta / state.zoom;
                self.last_mouse_position = event.position;
            });

            cx.notify();
        }
    }

    fn on_scrool_weel(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let scroll = event.delta.pixel_delta(px(16.)).y;

        //todo keep mouse position while scrolling
        //todo - scroll based on scroll amount
        if scroll > px(0.0) {
            self.render_state.update(cx, |state, _| {
                state.zoom *= 1.1;
            });
        } else if scroll < px(0.0) {
            self.render_state.update(cx, |state, _| {
                state.zoom /= 1.1;
            });
        }

        cx.notify();
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
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scrool_weel))
            .overflow_hidden()
            .size_full()
            .bg(rgb(0x606060))
    }
}
