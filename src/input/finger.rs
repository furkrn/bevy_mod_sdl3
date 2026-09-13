use bevy_ecs::entity::Entity;
use bevy_input::touch::{ForceTouch, TouchPhase, TouchInput};
use bevy_math::Vec2;
use bevy_window::WindowEvent;
use sdl3::event::Event as SdlEvent;

use crate::{SDL_CONTEXT, SdlContext};

pub fn handle_finger_events(
    sdl_event: SdlEvent,
    window_id: u32,
    window_events: &mut Vec<WindowEvent>,
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    let touch_event = match sdl_event {
        SdlEvent::FingerMotion { x, y, finger_id, pressure, touch_id, .. } if is_not_pen_touch(touch_id) => 
            sdl_finger_to_touch_event(x, y, pressure, finger_id, TouchPhase::Moved, window),
        SdlEvent::FingerUp { x, y, finger_id, pressure, touch_id, .. } if is_not_pen_touch(touch_id) => 
            sdl_finger_to_touch_event(x, y, pressure, finger_id, TouchPhase::Ended, window),
        SdlEvent::FingerDown { x, y, finger_id, pressure, touch_id, .. } if is_not_pen_touch(touch_id) => 
            sdl_finger_to_touch_event(x, y, pressure, finger_id, TouchPhase::Started, window),
        _ => return,
    };

    window_events.push(touch_event.into());
}

pub fn is_not_pen_touch(touch_id: u64) -> bool {
    touch_id != sdl3::sys::pen::SDL_PEN_TOUCHID.0 as u64
}

#[inline]
pub(crate) const fn sdl_finger_to_touch_event(
    x: f32, y: f32,
    pressure: f32,
    id: u64,
    phase: TouchPhase,
    window: Entity,
) -> TouchInput {
    let force = if pressure != 0.0 {
        Some(ForceTouch::Normalized(pressure as f64))
    } else {
        None
    };

    TouchInput {
        phase,
        window,
        id,
        force,
        position: Vec2::new(x, y),
    }
}