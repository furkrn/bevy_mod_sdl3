use bevy_ecs::message::Message;
use sdl3::event::Event;

#[derive(Message)]
pub struct RawSdlEvent(pub Event);

impl From<Event> for RawSdlEvent {
    #[inline]
    fn from(value: Event) -> Self {
        Self(value)
    }
}