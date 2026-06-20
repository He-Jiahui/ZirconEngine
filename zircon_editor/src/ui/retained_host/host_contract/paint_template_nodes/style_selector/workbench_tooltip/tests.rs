use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn tooltip_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(23, 28, 32);
    node.label_color = Color::from_rgb_u8(168, 179, 184);
    node.icon_color = Color::from_rgb_u8(37, 156, 167);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(23, 28, 32, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(37, 45, 50, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(208, 217, 221, 255)));

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.title, PALETTE.text_disabled);
    assert_eq!(style.body, PALETTE.text_disabled);
    assert_eq!(style.arrow, PALETTE.surface_disabled);
    assert_eq!(style.icon, PALETTE.text_disabled);
}
