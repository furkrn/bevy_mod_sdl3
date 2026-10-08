### SDL3 plugin for Bevy
Replaces `winit` in favor of SDL3. It might be limited compared to the `bevy_winit` plugin. Currently, bevy v0.19.1 is used.

Only proper use case I've found is for detecting pressure on a Wacom pen. Since [winit didn't implemented this yet](https://github.com/rust-windowing/winit/pull/2396).

### Getting started
You may want to see how you would link SDL3, first. Check `sdl3` crate's linkage documentation for this. [Same flags are used.](https://docs.rs/sdl3/latest/sdl3/#feature-flags)

A simple running example is [provided](./examples/simple.rs). As seen, you need to add the `Sdl3Plugin` and disable the `WinitPlugin` in order to use it properly.

```rs
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

```
