use super::model::WorkbenchSegmentedControlKind;
use super::palette::workbench_segmented_control_palette_from_host;
use super::selection::select_workbench_segmented_control_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostControlMetrics, METRICS, PALETTE,
};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

use super::metrics::workbench_segmented_selector_metrics_from_host;

#[test]
fn segmented_control_palette_projects_from_host_palette() {
    let host_palette = current_host_palette();
    let palette = workbench_segmented_control_palette_from_host(host_palette);

    assert_eq!(palette.idle_background, host_palette.surface);
    assert_eq!(palette.hot_background, host_palette.surface_hover);
    assert_eq!(palette.pressed_background, host_palette.surface_pressed);
    assert_eq!(palette.disabled_background, host_palette.surface_disabled);
    assert_eq!(palette.border, host_palette.border);
    assert_eq!(palette.active_border, host_palette.accent);
    assert_eq!(palette.focus_border, host_palette.focus_ring);
    assert_eq!(palette.disabled_border, host_palette.border_disabled);
    assert_eq!(palette.selected_background, host_palette.surface_pressed);
    assert_eq!(palette.selected_border, host_palette.accent);
    assert_eq!(palette.selected_underline, host_palette.accent);
    assert_eq!(palette.selected_text, host_palette.text);
    assert_eq!(palette.idle_text, host_palette.text_muted);
    assert_eq!(palette.disabled_text, host_palette.text_disabled);
    assert_eq!(palette.group_label, host_palette.text_muted);
}

#[test]
fn segmented_control_selector_metrics_project_from_host_metrics() {
    let host_metrics = HostControlMetrics {
        border_width: 2.0,
        tab_underline_height: 3.0,
        ..METRICS
    };

    let metrics = workbench_segmented_selector_metrics_from_host(host_metrics);

    assert_eq!(metrics.border_width, 2.0);
    assert_eq!(metrics.selected_underline_height, 3.0);
}

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

#[test]
fn focused_segmented_control_keeps_idle_background_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let segmented = select_workbench_segmented_control_style(
        &node,
        WorkbenchSegmentedControlKind::SegmentedControl,
    );

    assert_eq!(segmented.state, UiPainterResolvedState::Focused);
    assert_eq!(segmented.background, Some(PALETTE.surface));
    assert_eq!(segmented.border, Some(PALETTE.focus_ring));
}

#[test]
fn focused_hovered_segmented_control_keeps_hover_background_and_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.hovered = true;

    let segmented = select_workbench_segmented_control_style(
        &node,
        WorkbenchSegmentedControlKind::SegmentedControl,
    );

    assert_eq!(segmented.state, UiPainterResolvedState::Focused);
    assert_eq!(segmented.background, Some(PALETTE.surface_hover));
    assert_eq!(segmented.border, Some(PALETTE.focus_ring));
}

#[test]
fn focused_tab_keeps_declared_background_without_active_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let tab = select_workbench_segmented_control_style(&node, WorkbenchSegmentedControlKind::Tab);

    assert_eq!(tab.state, UiPainterResolvedState::Focused);
    assert_eq!(tab.background, None);
    assert_eq!(tab.border, None);
}
