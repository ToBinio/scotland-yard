use gpui::{App, ClickEvent, ElementId, IntoElement, Window, div, prelude::*, rgb};

use crate::sidebar::EventListener;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: String,
    listener: EventListener,
}

impl Button {
    pub fn new(
        id: impl Into<ElementId>,
        label: String,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label,
            listener: Box::new(listener),
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .id(self.id.clone())
            .child(self.label)
            .cursor_pointer()
            .bg(rgb(0x707070))
            .text_color(rgb(0xffffff))
            .hover(|style| style.bg(rgb(0x808080)))
            .active(|style| style.bg(rgb(0x909090)))
            .px_2()
            .py_1()
            .mx_2()
            .rounded_md()
            .on_click(self.listener)
    }
}
