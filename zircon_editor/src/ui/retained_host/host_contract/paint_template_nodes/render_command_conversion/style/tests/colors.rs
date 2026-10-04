use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn runtime_foreground_fallback_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];
    let style = UiResolvedStyle::default();

    assert_eq!(
        runtime_foreground_color_from_host(&style, palette),
        [10, 11, 12, 255]
    );
}

#[test]
fn runtime_foreground_declared_color_overrides_palette() {
    let palette = PALETTE;
    let style = UiResolvedStyle {
        foreground_color: Some("#123456".to_owned()),
        ..UiResolvedStyle::default()
    };

    assert_eq!(
        runtime_foreground_color_from_host(&style, palette),
        [0x12, 0x34, 0x56, 255]
    );
}
