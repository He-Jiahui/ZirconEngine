use super::model::WorkbenchSegmentedControlKind;
use super::selection::select_workbench_segmented_control_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn segmented_and_tab_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.checked = true;
    node.selected = true;
    node.button_style.loading = true;
    node.label_color = Color::from_rgb_u8(161, 172, 178);
    node.selected_segment_underline_height = 1.0;
    node.selected_segment_underline_color = Color::from_argb_u8(255, 53, 199, 208);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(29, 35, 39, 255)));

    let segmented = select_workbench_segmented_control_style(
        &node,
        WorkbenchSegmentedControlKind::SegmentedControl,
    );

    assert_eq!(segmented.state, UiPainterResolvedState::Loading);
    assert_eq!(segmented.background, Some(PALETTE.surface_disabled));
    assert_eq!(segmented.border, Some(PALETTE.border_disabled));
    assert_eq!(segmented.selected_surface, PALETTE.surface_disabled);
    assert_eq!(segmented.selected_border, PALETTE.border_disabled);
    assert_eq!(segmented.selected_underline, PALETTE.text_disabled);
    assert_eq!(segmented.selected_text, PALETTE.text_disabled);
    assert_eq!(segmented.idle_text, PALETTE.text_disabled);
    assert_eq!(segmented.group_label, PALETTE.text_disabled);

    let tab = select_workbench_segmented_control_style(&node, WorkbenchSegmentedControlKind::Tab);

    assert_eq!(tab.state, UiPainterResolvedState::Loading);
    assert_eq!(tab.background, Some(PALETTE.surface_disabled));
    assert_eq!(tab.border, None);
    assert_eq!(tab.selected_underline, PALETTE.text_disabled);
    assert_eq!(tab.selected_text, PALETTE.text_disabled);
    assert_eq!(tab.idle_text, PALETTE.text_disabled);
}

#[test]
fn selected_segment_uses_pressed_surface_and_accent_underline() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.checked = true;

    let segmented = select_workbench_segmented_control_style(
        &node,
        WorkbenchSegmentedControlKind::SegmentedControl,
    );

    assert_eq!(segmented.selected_surface, PALETTE.surface_pressed);
    assert_ne!(segmented.selected_surface, PALETTE.surface_selected);
    assert_eq!(segmented.selected_underline, PALETTE.accent);
    assert_eq!(segmented.selected_border_width, 0.0);
}
