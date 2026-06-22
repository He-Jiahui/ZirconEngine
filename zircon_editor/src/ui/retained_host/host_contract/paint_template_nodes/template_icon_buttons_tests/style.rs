use super::super::{icon_button_context, icon_button_style, IconButtonContext, ICON_PANEL_RADIUS};
use super::support::{icon_node, resolved_panel_surface, resolved_panel_surface_with_radius};
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
        Some([47, 70, 80, 255])
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
    node.corner_radius = 5.0;
    assert_eq!(
        icon_button_style(&node, IconButtonContext::Panel).radius,
        ICON_PANEL_RADIUS
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
        Some([239, 112, 102, 255])
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
    assert_eq!(style.background, Some([16, 60, 74, 255]));
    assert_eq!(style.border, Some([128, 234, 255, 255]));
    assert_eq!(style.glyph, [128, 234, 255, 255]);

    node.disabled = true;
    let disabled_style = icon_button_style(&node, icon_button_context(&node));
    assert_eq!(disabled_style.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled_style.background, None);
    assert_eq!(disabled_style.border, None);
    assert_eq!(disabled_style.border_width, 1.0);
    assert_eq!(disabled_style.glyph, [88, 101, 108, 255]);
}
