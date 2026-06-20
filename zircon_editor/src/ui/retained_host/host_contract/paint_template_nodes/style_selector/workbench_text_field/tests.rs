use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn text_field_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.validation_level = "error".into();
    node.button_style.loading = true;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(16, 22, 26, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

    let style = select_workbench_text_field_style(&node, true);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, WORKBENCH_TEXT_FIELD_DISABLED_SURFACE);
    assert_eq!(style.border, WORKBENCH_TEXT_FIELD_DISABLED_BORDER);
    assert_eq!(style.text, WORKBENCH_TEXT_FIELD_DISABLED_TEXT);
    assert_eq!(style.stepper, PALETTE.text_disabled);
}
