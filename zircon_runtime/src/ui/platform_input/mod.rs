//! Platform UI input adapters owned by the runtime UI subsystem.

mod keyboard_map;
mod winit_translation;

pub use winit_translation::{translate_winit_modifiers, translate_winit_window_event};
