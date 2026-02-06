use gpui::{App, ClickEvent, IntoElement, Window, div, prelude::*, rgb};

#[derive(IntoElement)]
pub struct Button {
    id: String,
    label: String,
    listener: Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
}

impl Button {
    pub fn new(
        id: String,
        label: String,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id,
            label,
            listener: Box::new(listener),
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .id(self.id)
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
