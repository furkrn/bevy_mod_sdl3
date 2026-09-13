use bevy_ecs::world::World;
use bevy_input::ButtonState;
use bevy_input::mouse::{MouseScrollUnit, 
    MouseButton, MouseButtonInput,
    MouseMotion, MouseWheel
};

use bevy_input::touch::TouchPhase;
use bevy_math::Vec2;
use bevy_window::{CursorMoved, WindowEvent as BevyWindowEvent};
use sdl3::event::Event as SdlEvent;
use sdl3::mouse::{MouseButton as SdlMouseButton, MouseWheelDirection};

use crate::window::try_update_window;
use crate::{SDL_CONTEXT, SdlContext};

#[inline]
pub const fn mouse_button(b: SdlMouseButton) -> MouseButton {
    match b {
        SdlMouseButton::Left => MouseButton::Left,
        SdlMouseButton::Middle => MouseButton::Middle,
        SdlMouseButton::Right => MouseButton::Right,
        SdlMouseButton::X1 => MouseButton::Back,
        SdlMouseButton::X2 => MouseButton::Forward,
        SdlMouseButton::Unknown => MouseButton::Other(0),
    }
}

pub const fn is_real_mouse(which: u32) -> bool {
    which != sdl3::sys::pen::SDL_PEN_MOUSEID.0 && which != sdl3::sys::touch::SDL_TOUCH_MOUSEID.0
}

pub fn handle_mouse_events(
    world: &mut World, 
    sdl_event: SdlEvent, 
    window_id: u32,
    window_events: &mut Vec<BevyWindowEvent>,
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    match sdl_event {
        SdlEvent::MouseMotion { x, y, xrel, yrel, which, .. } if is_real_mouse(which) => {
            window_events.push(MouseMotion {
                delta: Vec2::new(xrel, yrel),
            }.into());

            if let Some((delta, position)) = try_update_window(world, window, 
                |w| {
                    let logical_pos = Vec2::new(x, y);
                    let last_pos = w.cursor_position();
                    let delta = last_pos.map(|last_pos| logical_pos - last_pos);

                    w.set_cursor_position(Some(logical_pos));

                    (delta, logical_pos)
                }) {
                    window_events.push(CursorMoved {
                        window,
                        position,
                        delta,
                    }.into());
                }
        }
 
        SdlEvent::MouseButtonDown { mouse_btn, which, .. } if is_real_mouse(which) => {
            window_events.push(MouseButtonInput {
                button: mouse_button(mouse_btn),
                state: ButtonState::Pressed,
                window,
            }.into());
        }
        SdlEvent::MouseButtonUp { mouse_btn, which, .. } if is_real_mouse(which) => {
            window_events.push(MouseButtonInput {
                button: mouse_button(mouse_btn),
                state: ButtonState::Released,
                window,
            }.into());
        }
 
        SdlEvent::MouseWheel { x, y, direction, which, .. } if is_real_mouse(which)     => {
            let flip = matches!(direction, MouseWheelDirection::Flipped);
            let (x, y) = if flip { (-x, -y) } else { (x, y) };
            window_events.push(MouseWheel {
                unit: MouseScrollUnit::Line,
                x,
                y,
                window,
                phase: TouchPhase::Moved,
            }.into());
        },
        _ => (),
    }
}