use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn chip_background_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_hover = [10, 11, 12, 255];
    palette.accent = [20, 21, 22, 255];
    palette.accent_soft = [30, 31, 32, 255];
    palette.warning = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        chip_background_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );

    node.component_variant = "primary".into();
    assert_eq!(
        chip_background_color_from_host(&node, palette),
        Some([20, 21, 22, 255])
    );

    node.component_variant = "secondary".into();
    assert_eq!(
        chip_background_color_from_host(&node, palette),
        Some([30, 31, 32, 255])
    );

    node.component_variant = "warning".into();
    assert_eq!(
        chip_background_color_from_host(&node, palette),
        Some([40, 41, 42, 255])
    );
}

#[test]
fn chip_background_declared_override_and_outlined_state_stay_explicit() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();
    node.component_variant = "primary outlined".into();

    assert_eq!(chip_background_color_from_host(&node, palette), None);

    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(70, 71, 72, 255)));
    assert_eq!(
        chip_background_color_from_host(&node, palette),
        Some([70, 71, 72, 255])
    );
}
