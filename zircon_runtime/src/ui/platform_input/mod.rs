//! Platform UI input adapters owned by the runtime UI subsystem.

#[path = "keyboard_map.rs"]
mod keyboard_map;
#[path = "winit_translation.rs"]
mod winit_translation;

pub use winit_translation::{translate_winit_modifiers, translate_winit_window_event};
