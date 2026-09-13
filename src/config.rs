use bevy_ecs::resource::Resource;
use bevy_ecs::world::{FromWorld, World};
use core::time::Duration;

#[derive(Clone, Debug, Resource)]
pub struct SdlConfig {
    pub focused_mode: UpdateMode,
    pub unfocused_mode: UpdateMode,
}

impl FromWorld for SdlConfig {
    #[inline]
    fn from_world(_world: &mut World) -> Self {
        Self {
            focused_mode: UpdateMode::Continious,
            unfocused_mode: UpdateMode::Continious,
        }
    }
}

impl SdlConfig {
    #[inline]
    pub const fn get_update_mode(&self, focused: bool) -> UpdateMode {
        if focused {
            self.focused_mode
        } else {
            self.unfocused_mode
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateMode {
    Continious,
    Reactive(Duration),
}