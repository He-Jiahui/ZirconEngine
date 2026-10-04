use super::*;

#[test]
fn themes_map_to_runtime_values() {
    assert_eq!(window_theme(Theme::Light), ZR_RUNTIME_WINDOW_THEME_LIGHT_V1);
    assert_eq!(window_theme(Theme::Dark), ZR_RUNTIME_WINDOW_THEME_DARK_V1);
}
