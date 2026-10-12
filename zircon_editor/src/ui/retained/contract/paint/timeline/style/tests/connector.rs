use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn timeline_connector_projects_default_from_host_palette() {
    let mut palette = PALETTE;
    palette.separator_strong = [10, 11, 12, 255];
    let node = TemplatePaneNodeData::default();

    assert_eq!(
        timeline_connector_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );
}

#[test]
fn timeline_connector_declared_color_overrides_palette() {
    let mut palette = PALETTE;
    palette.separator_strong = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));

    assert_eq!(
        timeline_connector_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );
}
