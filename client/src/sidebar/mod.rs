use game::event::Role;
use gpui::{App, ClickEvent, Window};

mod button;
pub mod default;
pub mod lobby;

type EventListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Default, Clone)]
pub enum SidebarState {
    #[default]
    NONE,
    LOBBY,
    GAME(Role),
}
