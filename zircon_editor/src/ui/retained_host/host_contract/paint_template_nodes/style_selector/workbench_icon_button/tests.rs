use super::model::WorkbenchIconButtonContext;
use super::selection::select_workbench_icon_button_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn icon_button_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.checked = true;
    node.selected = true;
    node.button_style.loading = true;
    node.control_id = "WorkbenchDeleteIconButton".into();
    node.icon_name = "trash".into();
    node.validation_level = "danger".into();
    node.icon_color = Color::from_rgb_u8(239, 112, 102);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(63, 25, 28, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

    let panel = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Panel);

    assert_eq!(panel.state, UiPainterResolvedState::Loading);
    assert_eq!(panel.background, Some(PALETTE.surface_disabled));
    assert_eq!(panel.border, Some(PALETTE.border_disabled));
    assert_eq!(panel.border_width, 1.0);
    assert_eq!(panel.glyph, PALETTE.text_disabled);

    let toolbar = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);

    assert_eq!(toolbar.state, UiPainterResolvedState::Loading);
    assert_eq!(toolbar.background, None);
    assert_eq!(toolbar.border, None);
    assert_eq!(toolbar.border_width, 1.0);
    assert_eq!(toolbar.glyph, PALETTE.text_disabled);
}
