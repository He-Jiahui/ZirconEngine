use winit::window::Theme;
use zircon_runtime_interface::{ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1};

pub(in crate::entry::runtime_entry_app) fn window_theme(theme: Theme) -> u32 {
    match theme {
        Theme::Light => ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
        Theme::Dark => ZR_RUNTIME_WINDOW_THEME_DARK_V1,
    }
}

#[cfg(test)]
#[path = "tests/window.rs"]
mod tests;
