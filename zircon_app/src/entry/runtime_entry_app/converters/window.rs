use winit::window::Theme;
use zircon_runtime_interface::{ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1};

pub(in crate::entry::runtime_entry_app) fn window_theme(theme: Theme) -> u32 {
    match theme {
        Theme::Light => ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
        Theme::Dark => ZR_RUNTIME_WINDOW_THEME_DARK_V1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_map_to_runtime_values() {
        assert_eq!(window_theme(Theme::Light), ZR_RUNTIME_WINDOW_THEME_LIGHT_V1);
        assert_eq!(window_theme(Theme::Dark), ZR_RUNTIME_WINDOW_THEME_DARK_V1);
    }
}
