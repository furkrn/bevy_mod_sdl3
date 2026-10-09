#[cfg(feature = "clipboard")]
pub mod clipboard;

pub mod config;
pub mod display;
pub mod events;
pub mod input;
pub mod state;
pub mod systems;
pub mod window;

pub mod prelude {
    
}

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::NonSendMarker;
use bevy_window::{CursorOptions, ExitSystems, 
    RawHandleWrapperHolder, 
    Window, WindowCreated};

use core::cell::RefCell;
use sdl3::Sdl;

use crate::config::SdlConfig;
use crate::display::SdlDisplays;
use crate::events::RawSdlEvent;
use crate::input::SdlInputSystem;
use crate::state::{sdl_runner, ShouldSpawnWindows};
use crate::systems::*;
use crate::window::{SdlWindows, 
    WindowId, WindowMinimizeMaximizeEvent
};

pub type CreateWindowsParam<'w, 's> = (
    Commands<'w, 's>,
    Query<
    'w, 
    's, 
    (Entity, &'static mut Window, &'static CursorOptions, Option<&'static RawHandleWrapperHolder>)>,
    MessageWriter<'w, WindowCreated>,
    Res<'w, SdlDisplays>,
    NonSendMarker,
);

pub type CreateMonitorsParam<'w, 's> = (Commands<'w, 's>, ResMut<'w, SdlDisplays>, NonSendMarker);

pub struct SdlContext {
    sdl: Sdl,
    windows: SdlWindows,
}

impl SdlContext {
    fn init() {
        SDL_CONTEXT.with_borrow_mut(|o| {
            *o = Some(SdlContext {
                sdl: sdl3::init().unwrap(),
                windows: SdlWindows::default(),
            })
        })
    }

    fn get_window(id: u32) -> impl FnOnce(&Option<SdlContext>) -> Option<Entity> {
        move |sdl| {
            sdl.as_ref()
                .map(|t| &t.windows)
                .and_then(|t| t.get_window_entity(WindowId(id)))
        }
    }
}

thread_local! {
    pub(crate) static SDL_CONTEXT: RefCell<Option<SdlContext>> = RefCell::new(None);
}

// no input focus yet
#[derive(Default)]
pub struct Sdl3Plugin;

impl Plugin for Sdl3Plugin {
    fn build(&self, app: &mut App) {
        SdlContext::init();
        app.set_runner(sdl_runner)
            .init_resource::<ShouldSpawnWindows>()
            .init_resource::<SdlConfig>()
            .init_resource::<SdlDisplays>()
            .add_message::<RawSdlEvent>()
            .add_message::<WindowMinimizeMaximizeEvent>()
            .add_systems(Last, 
                (
                    update_windows,
                    changed_cursor_options,
                    despawn_windows.after(ExitSystems),
                    check_keyboard_focus_lost,
                ).chain())
            .add_plugins(SdlInputSystem)
            .add_observer(|_window: On<Add<Window>>, mut resource: ResMut<ShouldSpawnWindows>| -> Result {
                resource.0 = true;

                Ok(())
            },);

        #[cfg(feature = "clipboard")]
        {
            use crate::clipboard::Clipboard;
            app.init_non_send::<Clipboard>();
        }
    }
}
