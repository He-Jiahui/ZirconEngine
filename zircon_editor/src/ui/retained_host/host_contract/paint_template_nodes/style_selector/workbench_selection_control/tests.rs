use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn selection_controls_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.checked = true;
    node.selected = true;
    node.hovered = true;
    node.pressed = true;
    node.drop_hovered = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(67, 216, 226);
    node.label_color = Color::from_rgb_u8(131, 141, 148);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(32, 159, 168, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(34, 161, 170, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(255, 255, 255, 255)));

    for kind in [
        WorkbenchSelectionControlKind::Checkbox,
        WorkbenchSelectionControlKind::Radio,
        WorkbenchSelectionControlKind::Toggle,
    ] {
        let style = select_workbench_selection_control_style(&node, kind);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, PALETTE.surface_disabled);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.thumb, PALETTE.text_disabled);
        assert_eq!(style.accent, PALETTE.text_disabled);
        assert_eq!(style.text, PALETTE.text_disabled);
        assert_eq!(style.label, PALETTE.text_disabled);
    }
}

#[test]
fn selection_controls_checked_state_uses_low_emphasis_markers() {
    let mut node = TemplatePaneNodeData::default();
    node.checked = true;
    node.selected = true;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(49, 191, 201, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(255, 255, 255, 255)));

    let checkbox =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Checkbox);
    let radio =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Radio);
    let toggle =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Toggle);

    assert_eq!(checkbox.surface, PALETTE.surface_selected);
    assert_eq!(checkbox.border, PALETTE.accent);
    assert_eq!(radio.surface, PALETTE.surface_pressed);
    assert_eq!(radio.border, PALETTE.separator_strong);
    assert_eq!(toggle.surface, PALETTE.surface_selected);
    assert_eq!(toggle.border, PALETTE.accent);
    assert_eq!(toggle.thumb, PALETTE.text);
}
