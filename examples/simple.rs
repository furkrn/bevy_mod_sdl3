use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_mod_sdl3::Sdl3Plugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build()
                .disable::<WinitPlugin>(),
            Sdl3Plugin,
        ))
        .run();
}