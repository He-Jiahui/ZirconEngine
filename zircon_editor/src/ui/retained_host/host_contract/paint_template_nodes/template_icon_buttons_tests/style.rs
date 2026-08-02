use super::super::{IconButtonContext, icon_button_context, icon_button_style};
use super::support::{icon_node, resolved_panel_surface, resolved_panel_surface_with_radius};
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
use zircon_runtime_interface::ui::style::{ResolvedButtonStyle, UiPainterResolvedState};

#[test]
fn panel_icon_button_uses_declared_glyph_color() {
    let mut node = icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        38.0,
        38.0,
    );
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(152, 163, 168);

    assert_eq!(
        icon_button_style(&node, icon_button_context(&node)).glyph,
        [152, 163, 168, 255]
    );
}

#[test]
fn panel_icon_button_uses_declared_surface_and_border() {
    let mut node = icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        38.0,
        38.0,
    );
    node.button_style = resolved_panel_surface([39, 45, 49, 255], [23, 31, 38, 255]);

    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).background,
        Some([39, 45, 49, 255])
    );
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).border,
        Some([23, 31, 38, 255])
    );

    node.hovered = true;
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).state,
        UiPainterResolvedState::Hovered
    );
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).background,
        Some(PALETTE.surface_hover)
    );
}

#[test]
fn panel_icon_button_uses_declared_radius_before_panel_default() {
    let mut node = icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        38.0,
        38.0,
    );
    node.button_style =
        resolved_panel_surface_with_radius([39, 45, 49, 255], [23, 31, 38, 255], 10.0);

    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).radius,
        10.0
    );

    node.button_style = ResolvedButtonStyle::default();
    node.corner_radius = 3.0;
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).radius,
        METRICS.radius_control
    );

    node.corner_radius = 10.0;
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).radius,
        10.0
    );
}

#[test]
fn panel_danger_icon_button_honors_declared_border_before_error_fallback() {
    let mut node = icon_node(
        "WorkbenchMiniDelete",
        "zircon_editor_shell/controls/delete.svg",
        false,
        38.0,
        38.0,
    );
    node.validation_level = "danger".into();
    node.button_style = resolved_panel_surface([39, 45, 49, 255], [23, 31, 38, 255]);

    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).border,
        Some([23, 31, 38, 255])
    );

    node.button_style = ResolvedButtonStyle::default();
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).border,
        Some(PALETTE.error)
    );
}

#[test]
fn icon_button_style_selector_uses_shared_state_priority() {
    let mut node = icon_node(
        "WorkbenchToolMove",
        "zircon_editor_shell/toolbar/move.svg",
        true,
        40.0,
        40.0,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.background, Some(PALETTE.surface_pressed));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.glyph, PALETTE.focus_ring);

    node.disabled = true;
    let disabled_style = icon_button_style(&node, icon_button_context(&node));
    assert_eq!(disabled_style.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled_style.background, None);
    assert_eq!(disabled_style.border, None);
    assert_eq!(disabled_style.border_width, 1.0);
    assert_eq!(disabled_style.glyph, PALETTE.text_disabled);
}

#[test]
fn focused_toolbar_icon_button_keeps_normal_tile_and_glyph_with_focus_border() {
    let mut node = icon_node(
        "WorkbenchToolMove",
        "zircon_editor_shell/toolbar/move.svg",
        false,
        40.0,
        40.0,
    );
    node.focused = true;

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn focused_hovered_toolbar_icon_button_still_uses_hover_fill() {
    let mut node = icon_node(
        "WorkbenchToolMove",
        "zircon_editor_shell/toolbar/move.svg",
        false,
        40.0,
        40.0,
    );
    node.focused = true;
    node.hovered = true;

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_hover));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn selected_focused_toolbar_icon_button_keeps_selected_surface_and_active_glyph() {
    let mut node = icon_node(
        "WorkbenchToolSelect",
        "zircon_editor_shell/toolbar/select.svg",
        true,
        40.0,
        40.0,
    );
    node.focused = true;

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_selected));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.glyph, PALETTE.focus_ring);
}

#[test]
fn focused_rail_icon_button_keeps_fillless_surface_and_muted_glyph() {
    let mut node = icon_node(
        "WorkbenchRailScene",
        "zircon_editor_shell/activity/play.svg",
        false,
        48.0,
        48.0,
    );
    node.focused = true;

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, None);
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, METRICS.border_width);
    assert_eq!(style.glyph, PALETTE.text_muted);
}

#[test]
fn asset_import_icon_button_uses_primary_accent_fill_with_theme_foreground() {
    let mut node = icon_node(
        "ImportModel",
        "editor_pages/asset_browser/import_pipeline/import.svg",
        false,
        80.0,
        28.0,
    );
    node.action_id = "workbench.asset.import_model".into();
    node.component_variant = "workbench-icon-button".into();

    let style = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(style.background, Some(PALETTE.accent));
    assert_eq!(style.border, Some(PALETTE.accent));
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.glyph, PALETTE.shell_background);

    node.hovered = true;
    let hovered = icon_button_style(&node, icon_button_context(&node));
    assert_eq!(hovered.background, Some(PALETTE.focus_ring));
    assert_eq!(hovered.border, Some(PALETTE.focus_ring));
    assert_eq!(hovered.glyph, PALETTE.shell_background);
}

#[test]
fn asset_import_icon_button_focus_keeps_primary_accent_fill() {
    let mut node = icon_node(
        "ImportModel",
        "editor_pages/asset_browser/import_pipeline/import.svg",
        false,
        80.0,
        28.0,
    );
    node.action_id = "workbench.asset.import_model".into();
    node.component_variant = "workbench-icon-button".into();
    node.focused = true;

    let focused = icon_button_style(&node, icon_button_context(&node));

    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.accent));
    assert_eq!(focused.border, Some(PALETTE.accent));
    assert_eq!(focused.glyph, PALETTE.shell_background);

    node.hovered = true;
    let focused_hovered = icon_button_style(&node, icon_button_context(&node));
    assert_eq!(focused_hovered.state, UiPainterResolvedState::Focused);
    assert_eq!(focused_hovered.background, Some(PALETTE.focus_ring));
    assert_eq!(focused_hovered.border, Some(PALETTE.focus_ring));
}

#[test]
fn top_toolbar_icon_button_uses_persistent_low_emphasis_tile() {
    let node = icon_node(
        "WorkbenchToolbarMenu",
        "zircon_editor_shell/toolbar/menu.svg",
        false,
        30.0,
        30.0,
    );
    let context = icon_button_context(&node);
    let style = icon_button_style(&node, context);

    assert_eq!(context, IconButtonContext::Toolbar);
    assert_eq!(style.background, Some(PALETTE.surface));
    assert_eq!(style.border, Some(PALETTE.border));
    assert_eq!(style.border_width, METRICS.border_width);
}

#[test]
fn dock_tab_close_button_uses_toolbar_context_without_persistent_panel_surface() {
    let node = icon_node("DockTabClose0", "close-outline", false, 20.0, 20.0);
    let context = icon_button_context(&node);
    let style = icon_button_style(&node, context);

    assert_eq!(context, IconButtonContext::Toolbar);
    assert_eq!(style.background, None);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}
