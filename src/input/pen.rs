use bevy_app::App;
use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::system::ResMut;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use bevy_input::{ButtonState, ButtonInput};
use bevy_input::touch::{ForceTouch, TouchInput, TouchPhase};
use bevy_math::Vec2;
use bevy_platform::collections::{HashMap, hash_map::Entry};
use bevy_window::WindowEvent as BevyWindowEvent;
use sdl3::event::Event;
use sdl3::pen::PenAxis as SdlPenAxis;

use crate::{SDL_CONTEXT, SdlContext};

pub(crate) fn add_pen_messages(app: &mut App) -> &mut App {
    app.add_message::<PenStateChanged>()
        .add_message::<PenButtonChanged>()
        .add_message::<PenMotionChanged>()
        .add_message::<PenProximityChanged>()
        .add_message::<PenAxisChanged>()
        .add_message::<PenEvent>()
        .init_resource::<Pens>()
        .init_resource::<ButtonInput<PenButton>>()
}

#[derive(Resource, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct PenButton(pub u8);

impl From<u8> for PenButton {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl PenButton {
    #[inline]
    pub const fn button_id(&self) -> u8 {
        self.0
    }
}

pub fn pen_button_input_system(
    mut button_message_reader: MessageReader<PenButtonChanged>,
    mut button_inputs: ResMut<ButtonInput<PenButton>>
) {
    button_inputs.bypass_change_detection().clear();
    for pen_button in button_message_reader.read() {
        match pen_button.state {
            ButtonState::Pressed => button_inputs.press(pen_button.button.into()),
            ButtonState::Released => button_inputs.release(pen_button.button.into()),
        }
    }
}

#[derive(Resource, Default)]
pub struct Pens {
    pressed_pens: HashMap<u32, Option<f32>>,
    release_count: u64,
}

impl Pens {
    pub fn press(&mut self, pen_id: u32, pressure: Option<f32>) {
        self.pressed_pens.insert(pen_id, pressure);
    }

    pub fn update_pressure(&mut self, pen_id: u32, pressure: Option<f32>) {
        if let Entry::Occupied(mut entry) = self.pressed_pens.entry(pen_id) {
            entry.insert(pressure);
        }
    }

    pub fn pressed(&self, pen_id: u32) -> bool {
        self.pressed_pens.contains_key(&pen_id)
    }

    pub fn pressure(&self, pen_id: u32) -> Option<f32> {
        self.pressed_pens.get(&pen_id).copied().flatten()
    }

    pub fn any_pressed(&self, pens: impl IntoIterator<Item = u32>) -> bool {
        pens.into_iter().any(|t| self.pressed(t))
    }

    pub fn all_pressed(&self, pens: impl IntoIterator<Item = u32>) -> bool {
        pens.into_iter().all(|t| self.pressed(t))
    }

    pub fn release(&mut self, pen_id: u32) {
        self.pressed_pens.remove(&pen_id);
        self.release_count += 1;
    }

    pub fn release_all(&mut self) {
        let len = self.pressed_pens.len();
        let _ = self.pressed_pens.drain();
        self.release_count += len as u64;
    }

    pub fn reset(&mut self) {
        self.pressed_pens.clear();
    }

    pub fn get_pressed(&self) -> impl Iterator<Item = u32> {
        self.pressed_pens.keys().copied()
    }
}

#[derive(Clone, Debug, Message)]
pub enum PenEvent {
    PenStateChanged(PenStateChanged),
    PenButtonChanged(PenButtonChanged),
    PenMotionChanged(PenMotionChanged),
    PenProximityChanged(PenProximityChanged),
    PenAxisChanged(PenAxisChanged),
}

impl From<PenStateChanged> for PenEvent {
    #[inline]
    fn from(value: PenStateChanged) -> Self {
        Self::PenStateChanged(value)
    }
}

impl From<PenButtonChanged> for PenEvent {
    #[inline]
    fn from(value: PenButtonChanged) -> Self {
        Self::PenButtonChanged(value)
    }
}

impl From<PenMotionChanged> for PenEvent {
    #[inline]
    fn from(value: PenMotionChanged) -> Self {
        Self::PenMotionChanged(value)
    }
}

impl From<PenProximityChanged> for PenEvent {
    #[inline]
    fn from(value: PenProximityChanged) -> Self {
        Self::PenProximityChanged(value)
    }
}

impl From<PenAxisChanged> for PenEvent {
    #[inline]
    fn from(value: PenAxisChanged) -> Self {
        Self::PenAxisChanged(value)
    }
}

#[derive(Clone, Debug, Message)]
pub struct PenStateChanged {
    pub state: PenState,
    pub window: Entity,
    pub which: u32,
    pub eraser: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PenState {
    Started,
    Motion,
    Released,
}

#[derive(Clone, Debug, Message)]
pub struct PenButtonChanged {
    pub state: ButtonState,
    pub button: u8,
    pub which: u32,
    pub window: Entity,
}

#[derive(Clone, Debug, Message)]
pub struct PenMotionChanged {
    pub vec2: Vec2,
    pub window: Entity,
    pub which: u32,
}

#[derive(Clone, Debug, Message)]
pub struct PenProximityChanged {
    pub proximity: PenProximity,
    pub window: Entity,
    pub which: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PenProximity {
    In,
    Out,
}

impl From<PenProximity> for bool {
    #[inline]
    fn from(value: PenProximity) -> Self {
        value == PenProximity::In
    }
}

#[derive(Clone, Debug, Message)]
pub struct PenAxisChanged {
    pub pen_axis: PenAxis,
    pub value: f32,
    pub which: u32,
    pub window: Entity,
    pub x: f32,
    pub y: f32,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum PenAxis {
    Pressure,
    XTilt,
    YTilt,
    Distance,
    Rotation,
    Slider,
    TangentialPressure,
    Count,
    Unknown = -1,
}

impl From<SdlPenAxis> for PenAxis {
    #[inline]
    fn from(value: SdlPenAxis) -> Self {
        match value {
            SdlPenAxis::Pressure => Self::Pressure,
            SdlPenAxis::XTilt => Self::XTilt,
            SdlPenAxis::YTilt => Self::YTilt,
            SdlPenAxis::Distance => Self::Distance,
            SdlPenAxis::Rotation => Self::Rotation,
            SdlPenAxis::Slider => Self::Slider,
            SdlPenAxis::TangentialPressure => Self::TangentialPressure,
            SdlPenAxis::Count => Self::Count,
            _ => Self::Unknown,
        }
    }
}

pub fn handle_pen_events(
    world: &mut World,
    sdl_event: Event, 
    window_id: u32, 
    pen_events: &mut Vec<PenEvent>,
    window_events: &mut Vec<BevyWindowEvent>
) {
    let Some(window) = SDL_CONTEXT.with_borrow(SdlContext::get_window(window_id)) else {
        bevy_log::warn!("Received an window event with id of {window_id} but this window is not present.");
        return;
    };

    let Some(mut pens) = world.get_resource_mut::<Pens>() else {
        bevy_log::warn!("Cannot handle pen events because pens resource is not registered!");
        return;
    };
    
    match sdl_event {
        Event::PenAxis { which, axis, value, x, y, .. } => {
            let pen_axis = axis.into();
            
            pen_events.push(PenAxisChanged {
                pen_axis,
                value,
                which,
                window,
                x, y
            }.into());

            if pen_axis == PenAxis::Pressure {
                pens.update_pressure(which, Some(value));
            }
            
        },
        Event::PenButtonDown { button, which, .. } => {
            pen_events.push(PenButtonChanged {
                state: ButtonState::Pressed,
                button,
                which,
                window
            }.into());
        },
        Event::PenButtonUp { button, which, .. } => {
            pen_events.push(PenButtonChanged {
                state: ButtonState::Released,
                button,
                which,
                window
            }.into());
        },
        Event::PenDown { which, x, y, eraser, .. } => {
            pen_events.push(PenStateChanged {
                state: PenState::Started,
                window,
                which,
                eraser,
            }.into());

            pens.press(which, None);
            let force = pens.pressure(which)
                .map(|t| ForceTouch::Normalized(t as f64));

            let id = pens.release_count + which as u64;
            window_events.push(TouchInput {
                force,
                phase: TouchPhase::Started,
                position: Vec2::new(x, y),
                id,
                window,
            }.into());
        },
        Event::PenMotion { x, y, which, .. } if pens.pressed(which) => {
            pen_events.push(PenMotionChanged {
                vec2: Vec2 { x, y },
                which,
                window
            }.into()); 

            let force = pens.pressure(which)
                .map(|t| ForceTouch::Normalized(t as f64));
            
            let id = pens.release_count + which as u64;
            window_events.push(TouchInput {
                force,
                phase: TouchPhase::Moved,
                position: Vec2::new(x, y),
                id,
                window,
            }.into());
        },
        Event::PenProximityIn { which, .. } => {
            pen_events.push(PenProximityChanged {
                proximity: PenProximity::In,
                which,
                window,
            }.into());
        },
        Event::PenProximityOut { which, .. } => {
            pen_events.push(PenProximityChanged {
                proximity: PenProximity::Out,
                which,
                window,
            }.into());

            pens.release(which);
        },
        Event::PenUp { which, x, y, eraser, .. } => {
            pen_events.push(PenStateChanged {
                state: PenState::Released,
                window,
                which,
                eraser,
            }.into());

            let id = pens.release_count + which as u64;
            window_events.push(TouchInput {
                force: None,
                phase: TouchPhase::Ended,
                position: Vec2::new(x, y),
                id,
                window,
            }.into());

            pens.release(which);
        },
        _ => ()
    }
}