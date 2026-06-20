use super::palette::WORKBENCH_DROPDOWN_DISABLED_SURFACE;
use super::selection::select_workbench_dropdown_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn dropdown_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.popup_open = true;
    node.selected = true;
    node.validation_level = "danger".into();
    node.button_style.loading = true;
    node.label_brightness = 1.8;
    node.value_color = Color::from_rgb_u8(205, 216, 221);
    node.icon_color = Color::from_rgb_u8(128, 234, 255);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(15, 101, 116, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

    let style = select_workbench_dropdown_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, WORKBENCH_DROPDOWN_DISABLED_SURFACE);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.chevron, PALETTE.text_disabled);
}
