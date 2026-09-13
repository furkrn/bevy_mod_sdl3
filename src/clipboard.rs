extern crate alloc;

use alloc::borrow::Cow;
use bevy_clipboard::{ClipboardError, ClipboardRead};
use sdl3::clipboard::ClipboardUtil as SdlClipboard;

use crate::SDL_CONTEXT;

pub struct Clipboard {
    clipboard_util: SdlClipboard,
}

impl Default for Clipboard {
    fn default() -> Self {
        SDL_CONTEXT.with_borrow(|sdl_context| {
            let sdl = sdl_context.as_ref()
                .map(|p| &p.sdl)
                .expect("SDL Context is not initialized");

            let clipboard_util = sdl.video()
                .map(|v| v.clipboard())
                .expect("Failed to initialize SDL clipboard.");

            Self { clipboard_util }
        })
    }
}

impl Clipboard {
    pub fn fetch_text(&self) -> ClipboardRead {
        ClipboardRead::Ready(
            self.clipboard_util.clipboard_text()
                .map_err(|_| ClipboardError::ClipboardNotSupported)
        )
    }

    pub fn set_text<'a, T: Into<Cow<'a, str>>>(&self, text: T) -> Result<(), ClipboardError> {
        self.clipboard_util.set_clipboard_text(&text.into())
            .map_err(|_| ClipboardError::ClipboardNotSupported)
    }
}