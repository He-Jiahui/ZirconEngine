use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn material_role_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    palette.shell_background = [20, 21, 22, 255];
    palette.surface_hover = [30, 31, 32, 255];
    palette.text_muted = [40, 41, 42, 255];
    palette.warning = [50, 51, 52, 255];

    assert_eq!(
        material_role_color_from_host("material.primary", palette),
        Some([10, 11, 12, 255])
    );
    assert_eq!(
        material_role_color_from_host("material.on_primary", palette),
        Some([20, 21, 22, 255])
    );
    assert_eq!(
        material_role_color_from_host("surface_hover", palette),
        Some([30, 31, 32, 255])
    );
    assert_eq!(
        material_role_color_from_host("muted", palette),
        Some([40, 41, 42, 255])
    );
    assert_eq!(
        material_role_color_from_host("warning", palette),
        Some([50, 51, 52, 255])
    );
    assert_eq!(material_role_color_from_host("unknown", palette), None);
}

#[test]
fn resolved_style_color_keeps_declared_and_inherit_semantics() {
    assert_eq!(
        resolved_style_color(Some(&UiStyleColor::Rgba(UiRgbaColor::from_u8(
            70, 71, 72, 255,
        )))),
        Some([70, 71, 72, 255])
    );
    assert_eq!(
        resolved_style_color(Some(&UiStyleColor::Transparent)),
        Some([0, 0, 0, 0])
    );
    assert_eq!(resolved_style_color(Some(&UiStyleColor::Inherit)), None);
    assert_eq!(resolved_style_color(None), None);
}
