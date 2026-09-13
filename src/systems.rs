use bevy_app::AppExit;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_ecs::system::NonSendMarker;
use bevy_input::ButtonState;
use bevy_input::keyboard::{Key, KeyCode, 
    KeyboardFocusLost, KeyboardInput
};

use bevy_platform::collections::HashMap;
use bevy_math::{IVec2, UVec2};
use bevy_window::{ClosingWindow, CursorOptions,
    HasWindows, Ime, Monitor, OnMonitor,
    PrimaryMonitor, RawHandleWrapper, VideoMode,
    Window, WindowEvent, WindowPosition,
    WindowCreated, WindowClosing, WindowClosed,
    WindowFocused, WindowMode, WindowResized,
    WindowScaleFactorChanged, WindowWrapper
};

use crate::{SDL_CONTEXT, CreateMonitorsParam, CreateWindowsParam};
use crate::display::{SdlDisplays, 
    get_refresh_rate_millihertz, select_display
};

use crate::window::{SyncSdlWindow, attempt_grab_sdl_window, 
    get_window_theme, react_to_resize
};

use sdl3::video::WindowPos;

#[derive(Clone, Component, Deref, DerefMut)]
pub struct CachedWindow(Window);

#[derive(Clone, Component, Deref, DerefMut)]
pub struct CachedCursorOptions(CursorOptions);

pub(crate) fn create_windows((mut commands,
    mut query,
    mut window_created_msg,
    displays,
    _non_send): CreateWindowsParam
) -> Result<(), BevyError> {
    SDL_CONTEXT.with_borrow_mut(|sdl| {
        let (sdl_context, windows) = sdl.as_mut()
            .map(|t| (&t.sdl, &mut t.windows))
            .expect("Failed to create SDL_CONTEXT.");

        for (entity, mut window, cursor_options, raw_window_handle_holder) in &mut query {
            if windows.get_window(entity).is_some() {
                continue
            }

            if let Some(window_theme) = get_window_theme() {
                window.window_theme = Some(window_theme);
            };

            let sdl_window = windows.create_window(&sdl_context, 
                entity, 
                cursor_options, 
                &window, 
                &displays)?;

            commands.entity(entity)
                .insert(CachedCursorOptions(cursor_options.clone()))
                .insert(CachedWindow(window.clone()))
                .insert(SdlWindowPressedKeys::default());

            if let Ok(handle_wrapper) = RawHandleWrapper::new(sdl_window) {
                commands.entity(entity)
                    .insert(handle_wrapper.clone());

                if let Some(handle_holder) = raw_window_handle_holder {
                    *handle_holder.0.lock().unwrap() = Some(handle_wrapper);
                }
            }

            window_created_msg.write(WindowCreated { window: entity });
        }

        Ok(())
    })
}

pub(crate) fn update_windows(
    mut commands: Commands,
    changed_windows: Query<(Entity, &mut Window, &mut CachedWindow, Option<&OnMonitor>), Changed<Window>>,
    displays: Res<SdlDisplays>,
    mut window_resized: MessageWriter<WindowResized>,
    mut window_event: MessageWriter<WindowEvent>,
    mut window_rescaled: MessageWriter<WindowScaleFactorChanged>,
    _non_send_marker: NonSendMarker,

) -> Result<(), BevyError> {
    SDL_CONTEXT.with_borrow_mut(|sdl| {
        let Some((sdl_context, windows)) = sdl.as_mut()
            .map(|t| (&t.sdl, &mut t.windows)) else {
                return Ok(())
            };

            for (entity, mut window, mut cache, monitor_relationship) in changed_windows {
                let Some(mut sdl_window) = windows.get_window(entity).map(|w| (*w).clone()) else {
                    continue;
                };

                if window.title != cache.title {
                    if let Err(e) = sdl_window.set_title(&window.title) {
                        bevy_log::warn!("Failed to set title for window {} : {e}", cache.title);
                    }
                }

                if window.mode != cache.mode {
                    let (fullscreen, monitor, video_mode) = match window.mode {
                        WindowMode::BorderlessFullscreen(monitor) => (true, Some(monitor), None),
                        WindowMode::Fullscreen(monitor, video_mode) => (true, Some(monitor), Some(video_mode)),
                        WindowMode::Windowed => (false, None, None),
                    };

                    if let Some(monitor) = monitor {
                        let primary_display = sdl_context.video()?.get_primary_display().ok();
                        if let Some(display) = crate::display::select_display(
                            &displays,
                            primary_display,
                            sdl_window.get_display().ok(),
                            &monitor,
                        ) {
                            if let Some(mut display_mode) = sdl_window.display_mode() {
                                display_mode.display = display;
                                if let Ok(display_mode) = crate::display::try_set_sdl_video_mode(display_mode, video_mode) {
                                    if let Err(error) = sdl_window.set_display_mode(display_mode) {
                                        bevy_log::warn!("Failed to set display mode for {}: {error}", window.title);
                                    }
                                }
                            }
                        }
                    }

                    if let Err(error) = sdl_window.set_fullscreen(fullscreen) {
                        bevy_log::warn!("Failed to set fullscreen mode for {}: {error}", window.title);
                    }
                }

                if window.position != cache.position {
                    let pos = match window.position {
                        WindowPosition::Automatic => None,
                        WindowPosition::Centered(monitor_selection) => {
                            let primary_display = sdl_context.video()
                                .and_then(|v| v.get_primary_display())
                                .ok();
                            
                            let maybe_display = select_display(&displays, 
                                primary_display, None, &monitor_selection);

                            if let Some(display) = maybe_display {
                                let display_mode = sdl_window.display_mode()
                                    .ok_or_else(|| "Failed to display mode.")
                                    .and_then(|mut display_mode| {
                                        display_mode.display = display;

                                        Ok(display_mode)
                                    });

                                if let Err(e) = display_mode.map(|d| sdl_window.set_display_mode(d)) {
                                    bevy_log::error!("Failed to update display mode for window {e}")
                                }

                                sdl_window.set_position(WindowPos::Centered, WindowPos::Centered);
                    
                                None
                            } else {
                                bevy_log::warn!("Couldn't retrieve the selected monitor. {monitor_selection:?}");
                                None
                            }
                        },
                        WindowPosition::At(pos) => {
                            Some((WindowPos::Positioned(pos.x as i32), WindowPos::Positioned(pos.y as i32)))    
                        },
                    };

                    if let Some((x, y)) = pos {
                        sdl_window.set_position(x, y);
                    }
                }

                if window.resolution != cache.resolution {
                    let width = window.resolution.physical_width();
                    let height = window.resolution.physical_height();
                    if let Err(error) = sdl_window.set_size(width, height) {
                        bevy_log::warn!("Failed to resize {}: {error}", window.title);
                    } else {
                        let event = react_to_resize(entity, &mut window, width, height);
                        
                        window_resized.write(event.clone());
                        window_event.write(event.into());
                    }

                    if window.scale_factor() != cache.scale_factor() {
                        let event = WindowScaleFactorChanged {
                            window: entity,
                            scale_factor: window.scale_factor() as f64,
                        };
                        window_rescaled.write(event.clone());
                        window_event.write(event.into());
                    }         
                }

                if window.physical_cursor_position() != cache.physical_cursor_position() {
                    if let Some(position) = window.physical_cursor_position() {
                        sdl_context.mouse().warp_mouse_in_window(
                            &sdl_window,
                            position.x as f32,
                            position.y as f32,
                        );
                    }
                }

                if window.decorations != cache.decorations {
                    if !sdl_window.set_bordered(window.decorations) {
                        bevy_log::warn!("Failed to set decorations for {}", window.title);
                    }
                }

                if window.resizable != cache.resizable {
                    bevy_log::warn!("Changing resizable state after creation is not supported by SDL3");
                }

                if window.enabled_buttons != cache.enabled_buttons {
                    bevy_log::warn!("Changing windows button states are not supported by SDL3");
                }

                if window.resize_constraints != cache.resize_constraints {
                    let constraints = window.resize_constraints.check_constraints();
                    if let Err(error) = sdl_window.set_minimum_size(
                        constraints.min_width as u32,
                        constraints.min_height as u32,
                    ) {
                        bevy_log::warn!("Failed to set minimum size for {}: {error}", window.title);
                    }

                    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                        if let Err(error) = sdl_window.set_maximum_size(
                            constraints.max_width as u32,
                            constraints.max_height as u32,
                        ) {
                            bevy_log::warn!("Failed to set maximum size for {}: {error}", window.title);
                        }
                    }
                }

                // also this is quite messy
                // would be better if we had a rework on it     
                // this has no way for removing that on monitor component
                if let Some(monitor_link) = monitor_relationship {
                    let current_display = sdl_window.get_display().ok();
                    if let Some(linked_display) = displays.find_entity(monitor_link.0)
                        && current_display.as_ref() != Some(&linked_display) {
                        bevy_log::warn!("Window {} is no longer on its linked monitor", window.title);
                    }
                } else if let Some(current_display) = sdl_window.get_display().ok()
                    && let Some((_, monitor_entity)) = displays.displays.iter()
                        .find(|(display, _)| display == &current_display) {
                    commands.entity(entity).insert(OnMonitor(*monitor_entity));
                }

                if let Some(maximized) = window.internal.take_maximize_request() {
                    if maximized {
                        sdl_window.maximize();
                    } else {
                        sdl_window.restore();
                    }
                }

                if let Some(minimized) = window.internal.take_minimize_request() {
                    if minimized {
                        sdl_window.minimize();
                    } else {
                        sdl_window.restore();
                    }
                }

                if window.internal.take_move_request() {
                    bevy_log::error!("Window move request is not implemented yet. Title of the window is {}", window.title)
                }

                if window.internal.take_resize_request().is_some() {
                    bevy_log::warn!("Window resize dragging is not supported by SDL3");

                }

                if window.focused != cache.focused {
                    if window.focused && !sdl_window.raise() {
                        bevy_log::warn!("Failed to focus window {}", window.title);
                    }
                }

                if window.window_level != cache.window_level {
                    bevy_log::warn!("Changing window level after creation is not supported by SDL3");
                }

                if window.transparent != cache.transparent {
                    bevy_log::warn!("Changing window transparency after creation is not supported by SDL3");
                }

                #[cfg(target_arch = "wasm32")]
                if window.canvas != cache.canvas {
                    window.canvas.clone_from(&cache.canvas);
                    bevy_log::warn!(
                        "Bevy currently doesn't support modifying the window canvas after initialization."
                    );
                }
                
                if window.ime_enabled != cache.ime_enabled {
                    match sdl_context.video().map(|t| t.text_input()) {
                        Ok(text_input_util) => {
                            let ime_event = if window.ime_enabled {
                                text_input_util.start(&sdl_window);
                                Ime::Enabled { window: entity }
                            } else {
                                text_input_util.stop(&sdl_window);
                                Ime::Disabled { window: entity }
                            };

                            window_event.write(ime_event.into());
                        },
                        Err(sdl_error) => {
                            bevy_log::warn!("An error received while trying to gather text input: {sdl_error}");
                        },
                    }
                }

                if window.ime_position != cache.ime_position {
                    bevy_log::warn!("Changing IME position after creation is not supported by SDL3");
                }

                if window.window_theme != cache.window_theme {
                    bevy_log::warn!("Changing window theme after creation is not supported by SDL3");
                }

                if window.visible != cache.visible {
                    if window.visible {
                        sdl_window.show();
                    } else {
                        sdl_window.hide();
                    }
                }

                #[cfg(target_os = "ios")]
                {
                    if window.recognize_pinch_gesture != cache.recognize_pinch_gesture {
                        bevy_log::error!("Pinch gesture recognization is not supported, {}", window.title);
                    }
                    if window.recognize_rotation_gesture != cache.recognize_rotation_gesture {
                        bevy_log::error!("Rotation gesture recognization is not supported, {}", window.title);
                    }
                    if window.recognize_doubletap_gesture != cache.recognize_doubletap_gesture {
                        bevy_log::error!("Double tap gesture recognization is not supported, {}", window.title);
                    }
                    if window.recognize_pan_gesture != cache.recognize_pan_gesture {
                        bevy_log::error!("Pan gesture recognization is not supported, {}", window.title);
                    }

                    if window.prefers_home_indicator_hidden != cache.prefers_home_indicator_hidden {
                        bevy_log::error!("Hiding home indicator is not supported, {}", window.title);
                    }
                    if window.prefers_status_bar_hidden != cache.prefers_status_bar_hidden {
                        bevy_log::error!("Hiding status bar is not supported, {}", window.title);
                    }
                    if window.preferred_screen_edges_deferring_system_gestures
                        != cache
                            .preferred_screen_edges_deferring_system_gestures
                    {
                        bevy_log::error!("Preferring screen edges deferring system gestures are not supported, {}", window.title)
                    }
                }

                **cache = window.clone();
            }

        Ok(())
    })
}

pub(crate) fn despawn_windows(
    closing: Query<Entity, With<ClosingWindow>>,
    mut closed_windows: RemovedComponents<Window>,
    window_entities: Query<Entity, With<Window>>,
    mut exit_reader: MessageReader<AppExit>,
    mut closing_window: MessageWriter<WindowClosing>,
    mut closed_window_writer: MessageWriter<WindowClosed>,
    mut windows_to_drop: Local<Vec<WindowWrapper<SyncSdlWindow>>>,
    _non_send_marker: NonSendMarker,
) -> Result<(), BevyError> {
    windows_to_drop.clear();
    for window in closing.iter() {
        closing_window.write(WindowClosing { 
            window
        });
    }
    SDL_CONTEXT.with_borrow_mut(|o| {
        let Some(windows) = o.as_mut()
            .map(|t| &mut t.windows) else {
                return
            };

        for window in closed_windows.read() {
            bevy_log::info!("Closing window {window}");

            if !window_entities.contains(window) {
                if let Some(sdl_window) = windows.remove_window(window) {
                    windows_to_drop.push(sdl_window);
                }
            }

            closed_window_writer.write(WindowClosed { window });
        }
    });

    if !exit_reader.is_empty() {
        exit_reader.clear();
        SDL_CONTEXT.with_borrow_mut(|o| {
            let Some(windows) = o.as_mut()
                .map(|t| &mut t.windows) else {
                    bevy_log::error!("Cannot despawn windows because of inexistant SDL_CONTEXT.");
                    return;
                };

            for window in window_entities.iter() {
                closing_window.write(WindowClosing { window });
                if let Some(sdl_window) = windows.remove_window(window) {
                    windows_to_drop.push(sdl_window);
                }

                closed_window_writer.write(WindowClosed { window });
            }
        })

    }

    Ok(())
}

pub(crate) fn create_monitors((mut commands, mut monitors, _non_send_marker): CreateMonitorsParam) -> Result<(), BevyError> {
    SDL_CONTEXT.with_borrow(|o| {
        let sdl_context = o.as_ref()
            .map(|t| &t.sdl)
            .ok_or(BevyError::error("SDL_CONTEXT is None, have you called SdlContext::init()?"))?;

        let video = sdl_context.video()?;
        let primary_display = video.get_primary_display()?;
        let mut seen_monitors = vec![false; monitors.displays.len()];

        'display_iter: for display in video.displays()? {
            for (idx, (d, _)) in monitors.displays.iter().enumerate() {
                if &display == d {
                    seen_monitors[idx] = true;
                    continue 'display_iter;
                }
            }
        
            let physical_position = display.get_bounds()
                .map(|rect| IVec2::new(rect.x, rect.y))?;

            let current_display_mode = display.get_mode()?;
            let refresh_rate_millihertz = get_refresh_rate_millihertz(current_display_mode);

            let monitor_entity = commands.spawn(
                Monitor {
                    name: display.get_name().ok(),
                    physical_height: current_display_mode.h as u32,
                    physical_width: current_display_mode.w as u32,
                    physical_position,
                    scale_factor: display.get_content_scale()
                        .map(|f| f as f64)
                        .unwrap_or(1.0),
                    
                    refresh_rate_millihertz,
                    video_modes: display.get_fullscreen_modes()
                        .iter()
                        .flatten()
                        .map(|display_mode| VideoMode {
                            physical_size: UVec2::new(display_mode.w as u32, display_mode.h as u32),
                            bit_depth: display_mode.format.bits_per_pixel() as u16,
                            refresh_rate_millihertz: get_refresh_rate_millihertz(*display_mode).unwrap_or(0),
                        })
                        .collect(),
                }
            ).id();

            if primary_display == display {
                commands.entity(monitor_entity).insert(PrimaryMonitor);
            }

            seen_monitors.push(true);
            monitors.displays.push((display, monitor_entity));
        }

        let mut idx = 0;
        monitors.displays.retain(|(_m, entity)| {
            if seen_monitors[idx] {
                idx += 1;
                true
            } else {
                commands.entity(*entity)
                    .remove::<HasWindows>()
                    .despawn();

                idx += 1;
                false
            }
        });

        Ok(())
    })
}

#[derive(Component, Default)]
pub struct SdlWindowPressedKeys(pub(crate) HashMap<KeyCode, Key>);

pub(crate) fn check_keyboard_focus_lost(
    mut window_focused_reader: MessageReader<WindowFocused>,
    mut keyboard_focus_lost_writer: MessageWriter<KeyboardFocusLost>,
    mut keyboard_input_writer: MessageWriter<KeyboardInput>,
    mut window_event_writer: MessageWriter<WindowEvent>,
    mut q_windows: Query<&mut SdlWindowPressedKeys>,
) {
    let mut focus_lost = vec![];
    let mut focus_gained = false;
    for e in window_focused_reader.read() {
        if e.focused {
            focus_gained = true
        } else {
            focus_lost.push(e.window);
        }
    }

    if !focus_gained {
        if !focus_lost.is_empty() {
            window_event_writer.write(WindowEvent::KeyboardFocusLost(KeyboardFocusLost));
            keyboard_focus_lost_writer.write(KeyboardFocusLost);
        }

        for window in focus_lost {
            let Ok(mut pressed_keys) = q_windows.get_mut(window) else {
                continue;
            };

            for (key_code, logical_key) in pressed_keys.0.drain() {
                let event = KeyboardInput {
                    key_code,
                    logical_key,
                    state: ButtonState::Released,
                    repeat: false,
                    window,
                    text: None,
                };

                window_event_writer.write(WindowEvent::KeyboardInput(event.clone()));
                keyboard_input_writer.write(event);
            }
        }
    }
}

pub(crate) fn changed_cursor_options(
    mut changed_windows: Query<(
        Entity,
        &Window,
        &mut CursorOptions,
        &mut CachedCursorOptions,
    ), Changed<CursorOptions>>,
    _non_send_marker: NonSendMarker,
) {
    SDL_CONTEXT.with_borrow_mut(|o| {
        let (sdl, windows) = o.as_mut()
            .map(|t| (&t.sdl, &mut t.windows))
            .expect("SDL_CONTEXT is not initialized, did you call SdlContext::init()?");

        for (entity, window, mut cursor_options, mut cached_cursor_options) in &mut changed_windows {
            let cursor_options = cursor_options.bypass_change_detection();
            let Some(mut sdl_window) = windows.get_window(entity).map(|t| (*t).clone()) else {
                bevy_log::warn!("received an window event from window with {} but the given window cannot be found", window.title);
                continue
            };

            if cursor_options.visible != cached_cursor_options.visible {
                let mouse = sdl.mouse();
                mouse.show_cursor(cursor_options.visible);
                cached_cursor_options.visible = cursor_options.visible;
            }

            if let Err(e) =  attempt_grab_sdl_window(&mut sdl_window, cursor_options.grab_mode) {
                bevy_log::warn!("Failed to set a proper grab mode for {} : {e}", window.title);
                cursor_options.grab_mode = cached_cursor_options.grab_mode;
            } else {
                cached_cursor_options.grab_mode = cursor_options.grab_mode;
            }

            if cursor_options.hit_test != cached_cursor_options.hit_test {
                bevy_log::warn!("SDL3 does not support disabling hit test right now, see https://github.com/libsdl-org/SDL/pull/14561");
            }
        }
    })

}