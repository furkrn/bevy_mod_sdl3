use bevy_derive::{Deref, DerefMut};
use bevy_ecs::entity::{Entity, EntityHashMap};
use bevy_ecs::prelude::*;
use bevy_math::IVec2;
use bevy_platform::collections::HashMap;
use bevy_window::{CursorEntered, 
    CursorGrabMode, CursorLeft, CursorOptions, 
    FileDragAndDrop, MonitorSelection, VideoModeSelection,
    Window, WindowCloseRequested, WindowEvent, 
    WindowFocused, WindowLevel, WindowMode, 
    WindowMoved, WindowOccluded, WindowPosition, 
    WindowResized, WindowTheme, WindowWrapper
};

use core::fmt::{Display, Formatter, Result as DisplayResult};
use core::marker::PhantomData;

use raw_window_handle::{DisplayHandle,
    HandleError, HasWindowHandle, 
    HasDisplayHandle, WindowHandle};

use sdl3::{Sdl, VideoSubsystem as SdlVideoSubsystem};
use sdl3::event::{Event, WindowEvent as SdlWindowEvent};
use sdl3::video::{SystemTheme as SdlSystemTheme, 
    Window as SdlWindow, WindowBuilder as SdlWindowBuilder, 
    WindowFlags as SdlWindowFlags
};

use crate::display::{SdlDisplays, 
    get_refresh_rate_millihertz, 
    select_display, try_set_sdl_video_mode};

use crate::{SDL_CONTEXT, SdlContext};

#[derive(Clone, Message)]
pub enum WindowMinimizeMaximizeEvent {
    Maximized(Entity),
    Minimized(Entity),
}

impl WindowMinimizeMaximizeEvent {
    #[inline]
    pub const fn get_window(&self) -> Entity {
        match self {
            Self::Maximized(w) => *w,
            Self::Minimized(w) => *w,
        }
    }

    #[inline]
    pub const fn is_minimized(&self) -> bool {
        matches!(self, Self::Minimized(_))
    }

    #[inline]
    pub const fn is_maximized(&self) -> bool {
        matches!(self, Self::Maximized(_))
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct WindowId(pub(crate) u32);

impl From<u32> for WindowId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Default)]
pub struct SdlWindows {
    pub windows: HashMap<WindowId, WindowWrapper<SyncSdlWindow>>,
    pub entity_to_sdl: EntityHashMap<WindowId>,
    pub sdl_to_entity: HashMap<WindowId, Entity>,
    _thread_safe_marker: PhantomData<*const ()>,
}

#[inline]
fn set_sdl_window_flags(
    mut window_builder: SdlWindowBuilder, 
    bevy_window: &Window, 
    video_mode: &mut Option<VideoModeSelection>,
    monitor_selection: &mut Option<MonitorSelection>,
) -> SdlWindowBuilder {
    let mut sdl_flags = window_builder.flags() | SdlWindowFlags::HIGH_PIXEL_DENSITY;
    
    match bevy_window.mode {
        WindowMode::BorderlessFullscreen(_) => {
            sdl_flags |= SdlWindowFlags::FULLSCREEN | SdlWindowFlags::BORDERLESS;
        }
        WindowMode::Windowed => {
            match bevy_window.position {
                WindowPosition::Automatic => {
                    window_builder.position_centered();
                },
                WindowPosition::Centered(monitor) => {
                    window_builder.position_centered();
                    *monitor_selection = Some(monitor);
                },
                WindowPosition::At(pos) => {
                    window_builder.position(pos.x, pos.y);
                }
            };
            
        }
        WindowMode::Fullscreen(monitor, video_mode_selection) => {
            *monitor_selection = Some(monitor);
            *video_mode = Some(video_mode_selection);
        }
    }

    if !bevy_window.decorations {
        sdl_flags |= SdlWindowFlags::BORDERLESS;
    }

    if bevy_window.transparent {
        sdl_flags |= SdlWindowFlags::TRANSPARENT;
    }

    if bevy_window.focused {
        sdl_flags |= SdlWindowFlags::INPUT_FOCUS | SdlWindowFlags::MOUSE_FOCUS
    }

    if bevy_window.resizable {
        sdl_flags |= SdlWindowFlags::RESIZABLE;
    }

    // ime???
    // sdl3 also has some sort of textinput event to handle
    // although im unsure about it.

    if let WindowLevel::AlwaysOnTop = bevy_window.window_level {
        sdl_flags |= SdlWindowFlags::ALWAYS_ON_TOP;
    }

    if !bevy_window.visible {
        sdl_flags |= SdlWindowFlags::HIDDEN
    }
    
    window_builder.set_flags(sdl_flags);
    window_builder
}

#[inline]
pub fn get_window_theme() -> Option<WindowTheme> {
    match SdlVideoSubsystem::get_system_theme() {
        SdlSystemTheme::Light => Some(WindowTheme::Light),
        SdlSystemTheme::Dark => Some(WindowTheme::Dark),
        SdlSystemTheme::Unknown => None,
    }
}

impl SdlWindows {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            entity_to_sdl: EntityHashMap::new(),
            sdl_to_entity: HashMap::new(),
            _thread_safe_marker: PhantomData,
        }
    }

    pub fn create_window(&mut self,
        sdl: &Sdl,
        entity: Entity,
        cursor_options: &CursorOptions,
        window: &Window,
        displays: &SdlDisplays,
    ) -> Result<&WindowWrapper<SyncSdlWindow>> {
        let video = sdl.video()?;
        let mouse = sdl.mouse();

        let sdl_window_builder = video.window(&window.title, 
            window.width() as u32, 
            window.height() as u32);

        let mut monitor_selection = None;
        let mut video_mode = None;
        let mut sdl_window = set_sdl_window_flags(sdl_window_builder, window, &mut video_mode, &mut monitor_selection)
            .build()?;

        let constraints = window.resize_constraints.check_constraints();
        if let Err(e) = sdl_window.set_minimum_size(constraints.min_width as u32, constraints.min_height as u32) {
            bevy_log::error!("Cannot set minimum size for {} : {e}", window.title);
        }

        if constraints.max_height.is_finite() && constraints.max_width.is_finite() &&
            let Err(e) = sdl_window.set_maximum_size(constraints.max_width as u32, constraints.max_height as u32) {
                bevy_log::error!("Cannot set maximum size for {} : {e}", window.title);
            }

        if monitor_selection.is_some() || video_mode.is_some() {
            let primary_display = video.get_primary_display()
                .ok();
            let display_mode = sdl_window.display_mode()
                .ok_or_else(|| "Failed to display mode.")
                .and_then(|mut display_mode| {
                    if let Some(display) = monitor_selection.and_then(|d| select_display(displays, primary_display, None, &d)) {
                        display_mode.display = display;
                    }

                    Ok(display_mode)
                })
                .and_then(|display_mode| {
                    try_set_sdl_video_mode(display_mode, video_mode)
                });

            if let Err(e) = display_mode.map(|d| sdl_window.set_display_mode(d)) {
                bevy_log::error!("Failed to update display mode for window {e}")
            }
        }

        if window.focused && !sdl_window.raise() {
            bevy_log::error!("Failed to raise window {}", window.title);
        }

        if let Err(e) = attempt_grab_sdl_window(&mut sdl_window, cursor_options.grab_mode) {
            bevy_log::error!("Failed to set a proper grab mode for {} : {e}", window.title)
        };

        mouse.show_cursor(cursor_options.visible);
        // either use transparent flag and SDL_SetWindowShape
        // or completely ignore it until that thing got merged in 3.6.0
        // I believe for my use case, we won't be needing disabled hit_test
        // do not mix this hit_test with SDL_SetCursorHitTest! They are very different thing
        // One is for ignoring cursor events other than is for dragging and resizing...
        if !cursor_options.hit_test {
            bevy_log::warn!("SDL3 does not support disabling hit test right now, see https://github.com/libsdl-org/SDL/pull/14561")
        }

        // TODO: accessibility?

        match DisplayInfo::try_new(&sdl_window, window) {
            Ok(display_info) => bevy_log::info!("{display_info}"),
            Err(e) => bevy_log::error!("Failed to retrieve display info {e}"),
        };

        let window_id = WindowId(sdl_window.id());

        self.entity_to_sdl.insert(entity, window_id);
        self.sdl_to_entity.insert(window_id, entity);

        Ok(self.windows.entry(window_id)
            .insert(WindowWrapper::new(SyncSdlWindow(sdl_window)))
            .into_mut())
    }

    pub fn get_window(&self, entity: Entity) -> Option<&WindowWrapper<SyncSdlWindow>> {
        self.entity_to_sdl.get(&entity)
            .and_then(|i| self.windows.get(i))
    }

    pub fn get_window_entity(&self, id: WindowId) -> Option<Entity> {
        self.sdl_to_entity.get(&id).cloned()
    }

    pub fn remove_window(&mut self, entity: Entity) -> Option<WindowWrapper<SyncSdlWindow>> {
        let sdl_id = self.entity_to_sdl.remove(&entity)?;
        self.sdl_to_entity.remove(&sdl_id);
        self.windows.remove(&sdl_id)
    }
}

pub(crate) fn attempt_grab_sdl_window(sdl_window: &mut SdlWindow, grab_mode: CursorGrabMode) -> Result<()> {
    let grab_mode_succeedded = match grab_mode {
        CursorGrabMode::None => sdl_window.set_mouse_grab(false),
        CursorGrabMode::Confined => sdl_window.set_mouse_grab(true),
        CursorGrabMode::Locked => {
            return Err("CursorGrabMode::Locked is not supported on SDL.".into());
        }
    };

    grab_mode_succeedded.ok_or_else(|| sdl3::get_error().into())
}

#[derive(Deref, DerefMut)]
pub struct SyncSdlWindow(pub(crate) SdlWindow);

impl HasWindowHandle for SyncSdlWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for SyncSdlWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

unsafe impl<'w> Send for SyncSdlWindow {}
unsafe impl<'w> Sync for SyncSdlWindow {}

#[derive(Component)]
pub enum DropState {
    Begin,
    Dropped(String),
}

pub(crate) fn handle_window_drop_events(
    world: &mut World, 
    sdl_event: Event, 
    window_id: u32,
    window_events: &mut Vec<WindowEvent>
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };
    
    match sdl_event {
        Event::DropBegin { .. } => {
            world.entity_mut(window)
                .insert(DropState::Begin);
        },
        Event::DropComplete { .. } => {
            match world.entity_mut(window).take::<DropState>() {
                Some(DropState::Begin) => {
                    window_events.push(FileDragAndDrop::HoveredFileCanceled { window }.into());
                },
                Some(DropState::Dropped(filename)) => {
                    window_events.push(FileDragAndDrop::HoveredFile { window, path_buf: filename.clone().into() }.into());
                    window_events.push(FileDragAndDrop::DroppedFile { window, path_buf: filename.into() }.into());
                }
                None => (),
            };
        },
        Event::DropFile { filename, .. } => {
            world.entity_mut(window)
                .modify_component(|t: &mut DropState| *t = DropState::Dropped(filename.into()));
        },
        Event::DropText { filename, .. } => {
            world.entity_mut(window)
                .modify_component(|t: &mut DropState| *t = DropState::Dropped(filename.into()));
        },
        _ => (),
    }
}

pub(crate) fn try_update_window<T>(world: &mut World, window: Entity, f: impl FnOnce(&mut Window) -> T) -> Option<T> {
    if let Some(mut window) = world.entity_mut(window).get_mut::<Window>() {
        Some((f)(&mut window))
    } else {
        None
    }
}

pub(crate) fn react_to_resize(window: Entity, bevy_window: &mut Window, width: u32, height: u32) -> WindowResized {
    bevy_window.resolution
        .set_physical_resolution(width, height);

    WindowResized {
        window,
        width: bevy_window.width(),
        height: bevy_window.height(),
    }
}

pub(crate) fn handle_window_events(
    world: &mut World, 
    window_id: u32, 
    win_event: SdlWindowEvent,
    window_events: &mut Vec<WindowEvent>,
    min_max_events: &mut Vec<WindowMinimizeMaximizeEvent>,
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    match win_event {
        SdlWindowEvent::Shown => {
            try_update_window(world, window, 
                |w| w.visible = true);
        },
        SdlWindowEvent::Hidden => {
            try_update_window(world, window, 
                |w| w.visible = false);
        },
        SdlWindowEvent::Exposed => {
            window_events.push(WindowOccluded {
                window,
                occluded: false,
            }.into())
        },
        SdlWindowEvent::Moved(x, y) => {
            try_update_window(world, window, 
                |w| w.position = WindowPosition::At(IVec2 { x, y }));

            window_events.push(WindowMoved {
                window,
                position: IVec2 { x, y }
            }.into());
        },
        SdlWindowEvent::Resized(w, h) => {
            let window_size_change = try_update_window(world, window,
                |win| {
                    win.resolution.set(w as f32, h as f32);
                    win.size()
                });

            if let Some(window_size) = window_size_change {
                window_events.push(WindowResized {
                    window,
                    width: window_size.x,
                    height: window_size.y,
                }.into());  
            }
        },
        SdlWindowEvent::PixelSizeChanged(pw, ph) => {
            let window_size_change = try_update_window(world, window, 
                |w| {
                    w.resolution.set_physical_resolution(pw as u32, ph as u32);
                    w.size()
                });

            if let Some(window_size) = window_size_change {
                window_events.push(WindowResized {
                    window,
                    width: window_size.x,
                    height: window_size.y,
                }.into())
            }
        },
        SdlWindowEvent::Minimized => {
            min_max_events.push(WindowMinimizeMaximizeEvent::Minimized(window));
        },
        SdlWindowEvent::Maximized => {
            min_max_events.push(WindowMinimizeMaximizeEvent::Maximized(window));
        },
        SdlWindowEvent::Occluded => {
            window_events.push(WindowOccluded {
                window,
                occluded: true,
            }.into());
        },
        SdlWindowEvent::MouseEnter => {
            window_events.push(CursorEntered {
                window
            }.into());
        },
        SdlWindowEvent::MouseLeave => {
            try_update_window(world, window, 
                |w| w.set_cursor_position(None));

            window_events.push(CursorLeft {
                window
            }.into());
        },
        SdlWindowEvent::FocusGained => {
            try_update_window(world, window, 
                |w| w.focused = true);

            window_events.push(WindowFocused {
                window,
                focused: true,
            }.into());
        },
        SdlWindowEvent::FocusLost => {
            try_update_window(world, window, 
                |w| w.focused = false);

            window_events.push(WindowFocused {
                window,
                focused: false,
            }.into());
        },
        SdlWindowEvent::CloseRequested => {
            window_events.push(WindowCloseRequested {
                window
            }.into());
        },
        _ => (),
    }
}

struct DisplayInfo {
    window_physical_resolution: (u32, u32),
    window_logical_resolution: (f32, f32),
    monitor_name: Option<String>,
    scale_factor: Option<f64>,
    refresh_rate_millihertz: Option<u32>,
}

impl DisplayInfo {
    fn try_new(sdl_window: &SdlWindow, bevy_window: &Window) -> Result<Self> {
        let (refresh_rate_millihertz, display) = if let Some(display_mode) = sdl_window.display_mode() {
            (get_refresh_rate_millihertz(display_mode), display_mode.display)
        } else {
            (None, sdl_window.get_display()?)
        };

        Ok(DisplayInfo {
            window_physical_resolution: 
                (bevy_window.resolution.physical_width(), 
                bevy_window.resolution.physical_height()),

            window_logical_resolution: 
                (bevy_window.resolution.width(),
                bevy_window.resolution.height()),

            monitor_name: display.get_name().ok(),
            scale_factor: Some(sdl_window.display_scale() as f64),
            refresh_rate_millihertz: refresh_rate_millihertz,
        })
    }
}

impl Display for DisplayInfo {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        write!(f, "Display information:")?;
        write!(
            f,
            "  Window physical resolution: {}x{}",
            self.window_physical_resolution.0, self.window_physical_resolution.1
        )?;
        write!(
            f,
            "  Window logical resolution: {}x{}",
            self.window_logical_resolution.0, self.window_logical_resolution.1
        )?;
        write!(
            f,
            "  Monitor name: {}",
            self.monitor_name.as_deref().unwrap_or("")
        )?;
        write!(f, "  Scale factor: {}", self.scale_factor.unwrap_or(0.))?;
        let millihertz = self.refresh_rate_millihertz.unwrap_or(0);
        let hertz = millihertz / 1000;
        let extra_millihertz = millihertz % 1000;
        write!(f, "  Refresh rate (Hz): {hertz}.{extra_millihertz:03}")?;
        Ok(())
    }
}

