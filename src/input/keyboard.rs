use bevy_ecs::world::World;
use bevy_input::ButtonState;
use bevy_input::keyboard::{Key, KeyCode, 
    KeyboardInput, NativeKeyCode
};

use bevy_window::{Ime, WindowEvent as BevyWindowEvent};
use sdl3::event::Event as SdlEvent;
use sdl3::keyboard::{Keycode as SdlKeycode, 
    Scancode as SdlScancode
};

use crate::{SDL_CONTEXT, SdlContext};
use crate::systems::SdlWindowPressedKeys;

// AI-GENERATED
#[inline]
pub const fn scancode_to_keycode(sc: SdlScancode) -> KeyCode {
    use SdlScancode as S;
    match sc {
        S::A => KeyCode::KeyA, S::B => KeyCode::KeyB, S::C => KeyCode::KeyC,
        S::D => KeyCode::KeyD, S::E => KeyCode::KeyE, S::F => KeyCode::KeyF,
        S::G => KeyCode::KeyG, S::H => KeyCode::KeyH, S::I => KeyCode::KeyI,
        S::J => KeyCode::KeyJ, S::K => KeyCode::KeyK, S::L => KeyCode::KeyL,
        S::M => KeyCode::KeyM, S::N => KeyCode::KeyN, S::O => KeyCode::KeyO,
        S::P => KeyCode::KeyP, S::Q => KeyCode::KeyQ, S::R => KeyCode::KeyR,
        S::S => KeyCode::KeyS, S::T => KeyCode::KeyT, S::U => KeyCode::KeyU,
        S::V => KeyCode::KeyV, S::W => KeyCode::KeyW, S::X => KeyCode::KeyX,
        S::Y => KeyCode::KeyY, S::Z => KeyCode::KeyZ,

        S::_1 => KeyCode::Digit1, S::_2 => KeyCode::Digit2, S::_3 => KeyCode::Digit3,
        S::_4 => KeyCode::Digit4, S::_5 => KeyCode::Digit5, S::_6 => KeyCode::Digit6,
        S::_7 => KeyCode::Digit7, S::_8 => KeyCode::Digit8, S::_9 => KeyCode::Digit9,
        S::_0 => KeyCode::Digit0,

        S::Return => KeyCode::Enter,
        S::Escape => KeyCode::Escape,
        S::Backspace => KeyCode::Backspace,
        S::Tab => KeyCode::Tab,
        S::Space => KeyCode::Space,

        S::Minus => KeyCode::Minus,
        S::Equals => KeyCode::Equal,
        S::LeftBracket => KeyCode::BracketLeft,
        S::RightBracket => KeyCode::BracketRight,
        S::Backslash => KeyCode::Backslash,
        S::NonUsHash => KeyCode::IntlBackslash,
        S::Semicolon => KeyCode::Semicolon,
        S::Apostrophe => KeyCode::Quote,
        S::Grave => KeyCode::Backquote,
        S::Comma => KeyCode::Comma,
        S::Period => KeyCode::Period,
        S::Slash => KeyCode::Slash,
        S::NonUsBackslash => KeyCode::IntlBackslash,

        S::CapsLock => KeyCode::CapsLock,

        S::F1 => KeyCode::F1, S::F2 => KeyCode::F2, S::F3 => KeyCode::F3,
        S::F4 => KeyCode::F4, S::F5 => KeyCode::F5, S::F6 => KeyCode::F6,
        S::F7 => KeyCode::F7, S::F8 => KeyCode::F8, S::F9 => KeyCode::F9,
        S::F10 => KeyCode::F10, S::F11 => KeyCode::F11, S::F12 => KeyCode::F12,
        S::F13 => KeyCode::F13, S::F14 => KeyCode::F14, S::F15 => KeyCode::F15,
        S::F16 => KeyCode::F16, S::F17 => KeyCode::F17, S::F18 => KeyCode::F18,
        S::F19 => KeyCode::F19, S::F20 => KeyCode::F20, S::F21 => KeyCode::F21,
        S::F22 => KeyCode::F22, S::F23 => KeyCode::F23, S::F24 => KeyCode::F24,

        S::PrintScreen => KeyCode::PrintScreen,
        S::ScrollLock => KeyCode::ScrollLock,
        S::Pause => KeyCode::Pause,
        S::Insert => KeyCode::Insert,
        S::Home => KeyCode::Home,
        S::PageUp => KeyCode::PageUp,
        S::Delete => KeyCode::Delete,
        S::End => KeyCode::End,
        S::PageDown => KeyCode::PageDown,

        S::Right => KeyCode::ArrowRight,
        S::Left => KeyCode::ArrowLeft,
        S::Down => KeyCode::ArrowDown,
        S::Up => KeyCode::ArrowUp,

        S::NumLockClear => KeyCode::NumLock,
        S::KpDivide => KeyCode::NumpadDivide,
        S::KpMultiply => KeyCode::NumpadMultiply,
        S::KpMinus => KeyCode::NumpadSubtract,
        S::KpPlus => KeyCode::NumpadAdd,
        S::KpEnter => KeyCode::NumpadEnter,
        S::Kp1 => KeyCode::Numpad1, S::Kp2 => KeyCode::Numpad2, S::Kp3 => KeyCode::Numpad3,
        S::Kp4 => KeyCode::Numpad4, S::Kp5 => KeyCode::Numpad5, S::Kp6 => KeyCode::Numpad6,
        S::Kp7 => KeyCode::Numpad7, S::Kp8 => KeyCode::Numpad8, S::Kp9 => KeyCode::Numpad9,
        S::Kp0 => KeyCode::Numpad0,
        S::KpPeriod => KeyCode::NumpadDecimal,
        S::KpEquals => KeyCode::NumpadEqual,
        S::KpComma => KeyCode::NumpadComma,

        S::Application => KeyCode::ContextMenu,
        S::Power => KeyCode::Power,
        S::Execute => KeyCode::Open,
        S::Help => KeyCode::Help,
        S::Select => KeyCode::Select,
        S::Stop => KeyCode::MediaStop,
        S::Again => KeyCode::Again,
        S::Undo => KeyCode::Undo,
        S::Cut => KeyCode::Cut,
        S::Copy => KeyCode::Copy,
        S::Paste => KeyCode::Paste,
        S::Find => KeyCode::Find,

        S::Mute => KeyCode::AudioVolumeMute,
        S::VolumeUp => KeyCode::AudioVolumeUp,
        S::VolumeDown => KeyCode::AudioVolumeDown,

        S::LCtrl => KeyCode::ControlLeft,
        S::LShift => KeyCode::ShiftLeft,
        S::LAlt => KeyCode::AltLeft,
        S::LGui => KeyCode::SuperLeft,
        S::RCtrl => KeyCode::ControlRight,
        S::RShift => KeyCode::ShiftRight,
        S::RAlt => KeyCode::AltRight,
        S::RGui => KeyCode::SuperRight,

        S::Sleep => KeyCode::Sleep,
        S::MediaEject => KeyCode::Eject,
        S::MediaNextTrack => KeyCode::MediaTrackNext,
        S::MediaPreviousTrack => KeyCode::MediaTrackPrevious,
        S::MediaStop => KeyCode::MediaStop,
        S::MediaPlayPause => KeyCode::MediaPlayPause,
        S::MediaSelect => KeyCode::MediaSelect,
        S::AcSearch => KeyCode::BrowserSearch,
        S::AcHome => KeyCode::BrowserHome,
        S::AcBack => KeyCode::BrowserBack,
        S::AcForward => KeyCode::BrowserForward,
        S::AcStop => KeyCode::BrowserStop,
        S::AcRefresh => KeyCode::BrowserRefresh,
        S::AcBookmarks => KeyCode::BrowserFavorites,

        S::International1 => KeyCode::IntlRo,
        S::International3 => KeyCode::IntlYen,
        S::Lang1 => KeyCode::Lang1,
        S::Lang2 => KeyCode::Lang2,
        _ => KeyCode::Unidentified(NativeKeyCode::Unidentified),
    }
}

// AI-GENERATED
#[inline]
pub const fn keycode_to_named_key(kc: SdlKeycode) -> Option<Key> {
    use bevy_input::keyboard::Key as K;
    use sdl3::keyboard::Keycode as C;
    Some(match kc {
        C::Return => K::Enter,
        C::Escape => K::Escape,
        C::Backspace => K::Backspace,
        C::Tab => K::Tab,
        C::CapsLock => K::CapsLock,
        C::F1 => K::F1, C::F2 => K::F2, C::F3 => K::F3, C::F4 => K::F4,
        C::F5 => K::F5, C::F6 => K::F6, C::F7 => K::F7, C::F8 => K::F8,
        C::F9 => K::F9, C::F10 => K::F10, C::F11 => K::F11, C::F12 => K::F12,
        C::PrintScreen => K::PrintScreen,
        C::ScrollLock => K::ScrollLock,
        C::Pause => K::Pause,
        C::Insert => K::Insert,
        C::Home => K::Home,
        C::PageUp => K::PageUp,
        C::Delete => K::Delete,
        C::End => K::End,
        C::PageDown => K::PageDown,
        C::Right => K::ArrowRight,
        C::Left => K::ArrowLeft,
        C::Down => K::ArrowDown,
        C::Up => K::ArrowUp,
        C::NumLockClear => K::NumLock,
        C::LCtrl | C::RCtrl => K::Control,
        C::LShift | C::RShift => K::Shift,
        C::LAlt | C::RAlt => K::Alt,
        C::LGui | C::RGui => K::Super,
        C::Application => K::ContextMenu,
        C::Help => K::Help,
        _ => return None,
    })
}

pub fn handle_keyboard_event(
    world: &mut World, 
    sdl_event: SdlEvent, 
    window_id: u32,
    window_events: &mut Vec<BevyWindowEvent>,
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    let mut window_entity = world.entity_mut(window);
    let Ok(mut pressed_keys) = window_entity.get_components_mut::<&mut SdlWindowPressedKeys>() else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    match sdl_event {
        SdlEvent::KeyDown { scancode, keycode, repeat, .. } => {
            let key_code = scancode.map(scancode_to_keycode)
                .unwrap_or(KeyCode::Unidentified(NativeKeyCode::Unidentified));

            let logical_key = keycode
                .and_then(keycode_to_named_key)
                .unwrap_or(Key::Unidentified(bevy_input::keyboard::NativeKey::Unidentified));

            pressed_keys.0.insert(key_code, logical_key.clone());
            window_events.push(KeyboardInput {
                key_code,
                logical_key,
                state: ButtonState::Pressed,
                text: None,
                repeat,
                window,
            }.into());
        }
        SdlEvent::KeyUp { scancode, keycode, repeat, .. } => {
            let key_code = scancode.map(scancode_to_keycode)
                .unwrap_or(KeyCode::Unidentified(NativeKeyCode::Unidentified));
            
            let logical_key = keycode
                .and_then(keycode_to_named_key)
                .unwrap_or(Key::Unidentified(bevy_input::keyboard::NativeKey::Unidentified));
            
            pressed_keys.0.remove(&key_code);
            window_events.push(KeyboardInput {
                key_code,
                logical_key,
                state: ButtonState::Released,
                text: None,
                repeat,
                window,
            }.into());
        },
        SdlEvent::TextInput { text, .. } => {
            window_events.push(Ime::Commit {
                window,
                value: text
            }.into());
        },
        SdlEvent::TextEditing { text, start, length, .. } => {
            window_events.push(Ime::Preedit {
                cursor: Some((start as usize, length as usize)),
                value: text,
                window
            }.into());
        },
        _ => (),
    }
}