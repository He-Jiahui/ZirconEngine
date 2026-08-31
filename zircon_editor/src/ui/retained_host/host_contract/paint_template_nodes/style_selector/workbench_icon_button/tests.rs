use super::model::WorkbenchIconButtonContext;
use super::palette::workbench_icon_button_palette_from_host;
use super::selection::{
    icon_border_width_from_host, icon_radius_from_host, select_workbench_icon_button_style,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, METRICS, PALETTE,
};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{
    ButtonColor, UiPainterResolvedState, UiRgbaColor, UiStyleColor,
};

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
    assert_eq!(toolbar.border_width, 0.0);
    assert_eq!(toolbar.glyph, PALETTE.text_disabled);
}

#[test]
fn icon_button_metrics_project_from_host_control_metrics() {
    let mut metrics = METRICS;
    metrics.border_width = 2.0;
    metrics.radius_control = 6.0;

    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchToolbarMenu".into();

    assert_eq!(
        icon_border_width_from_host(
            &node,
            WorkbenchIconButtonContext::Toolbar,
            UiPainterResolvedState::Normal,
            metrics,
        ),
        0.0,
        "Starship toolbar buttons stay quiet until keyboard focus or pointer interaction"
    );
    assert_eq!(
        icon_radius_from_host(&node, WorkbenchIconButtonContext::Rail, metrics),
        6.0
    );
}

#[test]
fn toolbar_icon_button_uses_starship_quiet_simple_button_chrome() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchToolbarMenu".into();

    let normal = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);
    assert_eq!(normal.state, UiPainterResolvedState::Normal);
    assert_eq!(normal.background, None);
    assert_eq!(normal.border, None);
    assert_eq!(normal.border_width, 0.0);

    node.hovered = true;
    let hovered = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);
    assert_eq!(hovered.state, UiPainterResolvedState::Hovered);
    assert_eq!(hovered.background, Some(PALETTE.surface_hover));
    assert_eq!(hovered.border, None);
    assert_eq!(hovered.border_width, 0.0);

    node.focused = true;
    let focused_hovered =
        select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);
    assert_eq!(focused_hovered.background, Some(PALETTE.surface_hover));
    assert_eq!(focused_hovered.border, None);
    assert_eq!(focused_hovered.border_width, 0.0);

    node.hovered = false;
    node.focused = false;
    node.pressed = true;
    let pressed = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_pressed));
    assert_eq!(pressed.border, None);
    assert_eq!(pressed.border_width, 0.0);

    node.pressed = false;
    node.focused = true;
    let focused = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);
    assert_eq!(focused.background, None);
    assert_eq!(focused.border, Some(PALETTE.focus_ring));
    assert_eq!(focused.border_width, 1.0);
}

#[test]
fn toolbar_icon_button_normal_glyph_uses_semantic_button_tone() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchRunPlay".into();
    node.button_style.color = ButtonColor::Success;

    let normal = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);

    assert_eq!(normal.state, UiPainterResolvedState::Normal);
    assert_eq!(normal.glyph, PALETTE.success);
    assert_eq!(normal.background, None);
}

#[test]
fn selected_panel_icon_uses_neutral_border_and_local_active_glyph() {
    let node = TemplatePaneNodeData {
        selected: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Panel);

    assert_eq!(style.state, UiPainterResolvedState::Selected);
    assert_eq!(style.background, Some(PALETTE.surface_selected));
    assert_eq!(style.border, Some(PALETTE.border));
    assert_ne!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.glyph, PALETTE.accent);
    assert_ne!(style.glyph, PALETTE.focus_ring);
}

#[test]
fn pressed_panel_icon_uses_surface_feedback_without_a_focus_outline() {
    let node = TemplatePaneNodeData {
        pressed: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Panel);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.background, Some(PALETTE.surface_pressed));
    assert_eq!(style.border, Some(PALETTE.border));
    assert_ne!(style.border, Some(PALETTE.focus_ring));
}

#[test]
fn primary_import_icon_button_pure_focus_keeps_primary_surface_and_focus_ring() {
    let node = TemplatePaneNodeData {
        control_id: "ImportModel".into(),
        action_id: "workbench.asset.import_model".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.accent));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, METRICS.border_width);
    assert_eq!(style.glyph, PALETTE.shell_background);
}

#[test]
fn mixed_case_danger_identity_preserves_style() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchActionIcon".into(),
        icon_name: "TrAsH".into(),
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Panel);

    assert_eq!(style.background, Some(PALETTE.error_container));
    assert_eq!(style.glyph, PALETTE.error);
}

#[test]
fn icon_button_palette_projects_from_host_palette() {
    let host = current_host_palette();
    let palette = workbench_icon_button_palette_from_host(host);

    assert_eq!(palette.normal, host.text);
    assert_eq!(palette.muted, host.text_muted);
    assert_eq!(palette.panel_surface, host.surface_pressed);
    assert_eq!(palette.surface_hover, host.surface_hover);
    assert_eq!(palette.focus_ring, host.focus_ring);
    assert_eq!(palette.shell_background, host.shell_background);
}
