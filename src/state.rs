use bevy_app::PluginsState;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use bevy_window::{AppLifecycle, Window,
    WindowEvent as BevyWindowEvent
};

use sdl3::event::Event as SdlEvent;

use crate::config::{SdlConfig, UpdateMode};
use crate::events::RawSdlEvent;
use crate::input::handlers::*;
use crate::input::pen::PenEvent;

use crate::systems::*;
use crate::window::{WindowMinimizeMaximizeEvent,
    handle_window_events, handle_window_drop_events
};

use crate::{SDL_CONTEXT, CreateMonitorsParam, CreateWindowsParam};

#[derive(Resource)]
pub(crate) struct ShouldSpawnWindows(pub bool);

impl FromWorld for ShouldSpawnWindows {
    #[inline]
    fn from_world(_world: &mut World) -> Self {
        ShouldSpawnWindows(true)
    }
}

impl ShouldSpawnWindows {
    #[inline]
    pub fn spawning_windows(&mut self) -> bool {
        let spawn = self.0;
        if spawn {
            self.0 = false;
        }

        spawn
    }
}
pub struct SdlState {
    app: App,
    app_exit: Option<AppExit>,
    window_event_received: bool,
    raw_event_received: bool,
    lifecycle: AppLifecycle,
    previous_lifecycle: AppLifecycle,
    bevy_window_events: Vec<BevyWindowEvent>,
    raw_sdl_events: Vec<RawSdlEvent>,
    min_max_events: Vec<WindowMinimizeMaximizeEvent>,
    system_state: SystemState<(Query<'static, 'static, 
        &'static mut Window>, ResMut<'static, ShouldSpawnWindows>)>,

    pen_events: Vec<PenEvent>,
    pen_event_received: bool,
}

impl SdlState {
    fn new(mut app: App) -> Self {
        let system_state = SystemState::from_world(app.world_mut());

        Self {
            app,
            app_exit: None,
            window_event_received: false,
            raw_event_received: false,
            lifecycle: AppLifecycle::Idle,
            previous_lifecycle: AppLifecycle::Idle,
            bevy_window_events: vec![],
            raw_sdl_events: vec![],
            min_max_events: vec![],
            system_state,
            pen_events: vec![],
            pen_event_received: false,
        }
    }

    fn world(&mut self) -> &World {
        self.app.world()
    }

    fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    fn forward_pen_events(&mut self) {
        let pen_events = self.pen_events.drain(..).collect::<Vec<_>>();
        let world = self.world_mut();

        for pen_event in pen_events.iter() {
            match pen_event.clone() {
                PenEvent::PenAxisChanged(e) => {
                    world.write_message(e);
                },
                PenEvent::PenButtonChanged(e) => {
                    world.write_message(e);
                },
                PenEvent::PenMotionChanged(e) => {
                    world.write_message(e);
                },
                PenEvent::PenStateChanged(e) => {
                    world.write_message(e);
                },
                PenEvent::PenProximityChanged(e) => {
                    world.write_message(e);
                }
            }
        }

        if !pen_events.is_empty() {
            world.resource_mut::<Messages<PenEvent>>()
                .write_batch(pen_events);
        }
    }

    fn forward_sdl_events(&mut self) {
        let sdl_events = self.raw_sdl_events.drain(..).collect::<Vec<_>>();
        let world = self.world_mut();

        if !sdl_events.is_empty() {
            world.resource_mut::<Messages<RawSdlEvent>>()
                .write_batch(sdl_events);
        }
    }

    fn forward_min_max_events(&mut self) {
        let min_max_events = self.min_max_events.drain(..).collect::<Vec<_>>();
        let world = self.world_mut();

        if !min_max_events.is_empty() {
            world.resource_mut::<Messages<WindowMinimizeMaximizeEvent>>()
                .write_batch(min_max_events);
        }
    }

    fn forward_bevy_window_events(&mut self) {
        let window_events = self.bevy_window_events.drain(..).collect::<Vec<_>>();
            let world = self.world_mut();

        for window_event in window_events.iter() {
            match window_event.clone() {
                BevyWindowEvent::AppLifecycle(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::CursorEntered(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::CursorLeft(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::CursorMoved(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::FileDragAndDrop(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::Ime(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::RequestRedraw(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowBackendScaleFactorChanged(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowCloseRequested(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowCreated(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowDestroyed(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowFocused(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowMoved(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowOccluded(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowResized(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowScaleFactorChanged(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::WindowThemeChanged(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::MouseButtonInput(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::MouseMotion(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::MouseWheel(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::PinchGesture(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::RotationGesture(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::DoubleTapGesture(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::PanGesture(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::TouchInput(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::KeyboardInput(e) => {
                    world.write_message(e);
                }
                BevyWindowEvent::KeyboardFocusLost(e) => {
                    world.write_message(e);
                }
            }
        }

        if !window_events.is_empty() {
            world.resource_mut::<Messages<BevyWindowEvent>>()
                .write_batch(window_events);
        }
    }

    fn reset_on_update(&mut self) {
        self.raw_event_received = false;
        self.pen_event_received = false;
        self.window_event_received = false;
    }

    fn try_spawning_windows(&mut self) {
        let world = self.app.world_mut();
        let Ok((_, mut spawn_state)) = self.system_state.get_mut(world) else {
            return
        };

        if spawn_state.spawning_windows() {
            let mut system_param = SystemState::<CreateWindowsParam>::new(world);
            if let Err(e) = create_windows(system_param.get_mut(world).unwrap()) {
                bevy_log::warn!("Window creation returned an error : {e}");
            }

            system_param.apply(world);
        }
    }

    fn update_monitors(&mut self) {
        let world = self.world_mut();
        
        let mut system_state = SystemState::<CreateMonitorsParam>::new(world);
        if let Err(e) = create_monitors(system_state.get_mut(world).unwrap()) {
            bevy_log::warn!("create_monitors system returned an error : {e}")
        }

        system_state.apply(world);
    }

    fn run_app_update(&mut self) {
        self.reset_on_update();

        self.forward_bevy_window_events();
        self.forward_pen_events();
        self.forward_sdl_events();
        self.forward_min_max_events();

        self.try_spawning_windows();

        if self.app.plugins_state() == PluginsState::Cleaned {
            self.app.update();
        }
    }

    fn should_update(&mut self) -> bool {
        self.lifecycle.is_active()
    }

    fn update_app_lifecycle_from_sdl_event(&mut self, sdl_event: SdlEvent) {
        let lifecycle = match sdl_event {
            SdlEvent::AppDidEnterBackground { .. } => {
                AppLifecycle::WillSuspend
            },
            SdlEvent::AppWillEnterBackground { .. } => {
                AppLifecycle::Suspended
            },
            SdlEvent::AppDidEnterForeground { .. } => {
                AppLifecycle::Running
            },
            SdlEvent::AppWillEnterForeground { .. } => {
                AppLifecycle::WillResume
            },
            _ => return
        };

        self.update_app_lifecycle(lifecycle);
    }

    fn update_app_lifecycle(&mut self, lifecycle: AppLifecycle) {
        self.lifecycle = lifecycle;

        if self.lifecycle != self.previous_lifecycle {
            self.previous_lifecycle = self.lifecycle;
            self.bevy_window_events.push(self.lifecycle.into());
        }
    }

    fn sdl_event_loop(&mut self) -> Result<()> {
        let mut event_pump = SDL_CONTEXT.with_borrow_mut(|t| 
            t.as_mut().unwrap().sdl.event_pump())?;

        let mut system_state = SystemState::<(Res<SdlConfig>, Query<(Entity, &Window)>)>::new(self.world_mut());

        self.update_monitors();
        self.update_app_lifecycle(AppLifecycle::Running);
    
        'running: loop {
            let (config, windows) = system_state.get(self.world())
                .unwrap();

            let focused = windows.iter().any(|w| w.1.focused);
            let update_mode = config.get_update_mode(focused);

            let mut window_events = vec![];
            let mut pen_events = vec![];
            let mut min_max_events = vec![];
            
            let event_iter: &mut dyn Iterator<Item = SdlEvent> = match update_mode {
                UpdateMode::Continious => &mut event_pump.poll_iter(),
                UpdateMode::Reactive(d) => &mut event_pump.wait_timeout_iter(d),
            };
            
            for event in event_iter {
                if !event.is_unknown() {
                    self.raw_event_received = true;
                    self.raw_sdl_events.push(event.clone().into());
                }

                match event {
                    SdlEvent::Quit { .. } => {
                        let world = self.world_mut();
                        world.write_message(AppExit::Success);
                    },
                    SdlEvent::Window { window_id, win_event, .. } => {
                        self.window_event_received = true;
                        let world = self.world_mut();
                        handle_window_events(world, window_id, win_event, &mut window_events, &mut min_max_events)
                    },

                    SdlEvent::Display { .. } => {
                        self.update_monitors()
                    },

                    e @ SdlEvent::AppDidEnterBackground { .. } |
                    e @ SdlEvent::AppWillEnterBackground { .. } |
                    e @ SdlEvent::AppDidEnterForeground { .. } |
                    e @ SdlEvent::AppWillEnterForeground { .. } => {
                        self.update_app_lifecycle_from_sdl_event(e);
                    },

                    e @ SdlEvent::DropBegin { window_id, .. } | 
                    e @ SdlEvent::DropComplete { window_id, .. } | 
                    e @ SdlEvent::DropFile { window_id, .. } |
                    e @ SdlEvent::DropText { window_id, ..  } => {
                        self.window_event_received = true;
                        handle_window_drop_events(self.world_mut(), e, window_id, &mut window_events)
                    }

                    e @ SdlEvent::FingerDown { window_id, .. } | 
                    e @ SdlEvent::FingerUp { window_id, .. } | 
                    e @ SdlEvent::FingerMotion { window_id, .. } => {
                        self.window_event_received = true;
                        handle_finger_events(e, window_id, &mut window_events)
                    }

                    e @ SdlEvent::MouseButtonDown { window_id, .. } | 
                    e @ SdlEvent::MouseButtonUp { window_id, .. } |
                    e @ SdlEvent::MouseWheel { window_id, .. } | 
                    e @ SdlEvent::MouseMotion { window_id, .. } => {
                        self.window_event_received = true;
                        handle_mouse_events(self.world_mut(), e, window_id, &mut window_events);
                    }

                    e @ SdlEvent::KeyDown { window_id, .. } |
                    e @ SdlEvent::KeyUp { window_id, .. } | 
                    e @ SdlEvent::TextInput { window_id, .. } | 
                    e @ SdlEvent::TextEditing { window_id, .. } => {
                        self.window_event_received = true;
                        handle_keyboard_event(self.world_mut(), e, window_id, &mut window_events);
                    }

                    e @ SdlEvent::PenAxis { window, .. } |
                    e @ SdlEvent::PenButtonDown { window, .. } |
                    e @ SdlEvent::PenButtonUp { window, .. } |
                    e @ SdlEvent::PenDown { window, .. } |
                    e @ SdlEvent::PenMotion { window, .. } |
                    e @ SdlEvent::PenProximityIn { window, .. } |
                    e @ SdlEvent::PenProximityOut { window, .. } |
                    e @ SdlEvent::PenUp { window, .. } => {
                        self.pen_event_received = true;
                        self.window_event_received = true;
                        handle_pen_events(self.world_mut(),
                            e, window, 
                            &mut pen_events, 
                            &mut window_events);
                    }

                    _ => { }
                }
            }

            self.min_max_events.extend(min_max_events);
            self.pen_events.extend(pen_events);
            self.bevy_window_events.extend(window_events);

            if self.should_update() {
                self.run_app_update();
            }

            if let Some(exit) = self.app.should_exit() {
                self.app_exit = Some(exit);
            }

            if self.app_exit.is_some() {
                break 'running;
            }
        }

        Ok(())
    }
}

pub fn sdl_runner(mut app: App) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
            app.finish();
            app.cleanup();
    }

    let mut app_state = SdlState::new(app);

    if let Err(e) = app_state.sdl_event_loop() {
        bevy_log::error!("SDL event loop returned error : {e}");
    }

    SDL_CONTEXT.replace(None);

    app_state.app_exit.unwrap_or_else(|| {
        bevy_log::error!("Failed to receive an app exit code! This is a bug");
        AppExit::error()
    })
}
