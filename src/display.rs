use bevy_ecs::prelude::*;
use bevy_window::{MonitorSelection, VideoModeSelection};
use sdl3::video::{Display, DisplayMode};

#[derive(Default, Resource)]
pub struct SdlDisplays {
    pub(crate) displays: Vec<(Display, Entity)>,
}

impl SdlDisplays {
    pub fn nth(&self, n: usize) -> Option<Display> {
        self.displays.get(n).map(|(display, _)| display.clone())
    }

    pub fn find_entity(&self, entity: Entity) -> Option<Display> {
        self.displays.iter()
            .find(|(_, e)| *e == entity)
            .map(|(display, _)| display.clone())
    }
}

pub fn select_display(
    displays: &SdlDisplays,
    primary_monitor: Option<Display>,
    current_monitor: Option<Display>,
    monitor_selection: &MonitorSelection
) -> Option<Display> {
    match monitor_selection {
        MonitorSelection::Current => {
            if current_monitor.is_none() {
                bevy_log::warn!("Can't select current monitor on window creation or cannot find current monitor!");
            }

            current_monitor
        }
        MonitorSelection::Primary => primary_monitor,
        MonitorSelection::Index(n) => displays.nth(*n),
        MonitorSelection::Entity(entity) => displays.find_entity(*entity),
    }
}

pub(crate) fn try_set_sdl_video_mode<'err>(mut sdl_display_mode: DisplayMode, selection: Option<VideoModeSelection>) -> Result<DisplayMode, &'err str> {
    if let Some(VideoModeSelection::Specific(video_mode)) = selection {
        sdl_display_mode.w = video_mode.physical_size.x as i32;
        sdl_display_mode.h = video_mode.physical_size.y as i32;
        // set display mode what ever...
        //sdl_display_mode.refresh_rate = video_mode.refresh_rate_millihertz as f32;
        //sdl_display_mode.refresh_rate_numerator = (video_mode.refresh_rate_millihertz * 1000) as i32;
        //sdl_display_mode.refresh_rate_denominator = 1000;
        // unsure about anything else...
    }

    Ok(sdl_display_mode)
}

pub(crate) fn get_refresh_rate_millihertz(display_mode: DisplayMode) -> Option<u32> {
    if display_mode.refresh_rate_numerator > 0 && display_mode.refresh_rate_denominator > 0 {
        let numerator = display_mode.refresh_rate_numerator as u128;
        let denominator = display_mode.refresh_rate_denominator as u128;

        u32::try_from((numerator * 1000) / denominator).ok()
    } else if display_mode.refresh_rate.is_finite() && display_mode.refresh_rate > 0.0 {
        u32::try_from((display_mode.refresh_rate * 1000.0) as u64).ok()
    } else {
        None
    }
}