pub mod finger;
pub mod keyboard;
pub mod mouse;
pub mod pen;

use bevy_app::{App, Plugin};

pub mod handlers {
    pub use super::finger::handle_finger_events;
    pub use super::keyboard::handle_keyboard_event;
    pub use super::mouse::handle_mouse_events;
    pub use super::pen::handle_pen_events;
}

pub struct SdlInputSystem;

impl Plugin for SdlInputSystem {
    fn build(&self, app: &mut App) {
        pen::add_pen_messages(app);
    }
}