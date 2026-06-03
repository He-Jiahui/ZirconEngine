use super::super::super::data::TemplateNodeFrameData;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiPainterResolvedState, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

#[test]
fn icon_button_kind_matches_workbench_ids_and_excludes_status() {
    assert!(is_workbench_icon_button(&icon_node(
        "WorkbenchToolSelect",
        "zircon_editor_shell/toolbar/select.svg",
        false,
        40.0,
        40.0,
    )));
    assert!(is_workbench_icon_button(&icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        36.0,
        36.0,
    )));
    assert!(!is_workbench_icon_button(&icon_node(
        "WorkbenchStatusTarget",
        "target",
        false,
        34.0,
        30.0,
    )));
}

#[test]
fn selected_toolbar_icon_button_paints_active_surface_and_glyph() {
    let bytes = paint_template_nodes_for_test(
        64,
        56,
        model_rc(vec![TemplatePaneNodeData {
            selected: true,
            checked: true,
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: 8.0,
                width: 48.0,
                height: 40.0,
            },
            ..icon_node(
                "WorkbenchToolSelect",
                "zircon_editor_shell/toolbar/select.svg",
                true,
                48.0,
                40.0,
            )
        }]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 22, 16, 22, 24) > 0);
}

#[test]
fn normal_toolbar_icon_button_keeps_outer_background_clean_and_draws_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            34.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 48, 4, 4), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 12, 12, 18, 18) > 0);
}

#[test]
fn selected_rail_icon_button_paints_large_active_surface() {
    let bytes = paint_template_nodes_for_test(
        64,
        64,
        model_rc(vec![TemplatePaneNodeData {
            selected: true,
            checked: true,
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: 8.0,
                width: 48.0,
                height: 48.0,
            },
            ..icon_node(
                "WorkbenchRailScene",
                "zircon_editor_shell/activity/play.svg",
                true,
                48.0,
                48.0,
            )
        }]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 25, 20, 18, 24) > 0);
}

#[test]
fn panel_danger_icon_button_paints_surface_and_error_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchMiniDelete",
            "zircon_editor_shell/controls/delete.svg",
            false,
            36.0,
            36.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 14, 12, 20, 24) > 0);
}

#[test]
fn panel_icon_button_honors_declared_offset_and_icon_size() {
    let mut node = icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        38.0,
        38.0,
    );
    node.layout_offset_y = 1.35;
    node.value_number = 18.0;

    let paint_rect = icon_button_paint_rect(&node, &frame_rect(&node.frame));
    let glyph = icon_glyph_rect(&node, &paint_rect, icon_button_context(&node));

    assert!((paint_rect.y - 7.35).abs() < 0.001);
    assert!((glyph.x - 16.0).abs() < 0.001);
    assert!((glyph.y - 17.35).abs() < 0.001);
    assert_eq!(glyph.width, 18.0);
    assert_eq!(glyph.height, 18.0);
}

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

fn icon_node(
    control_id: &str,
    icon_name: &str,
    active: bool,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "IconButton".into(),
        icon_name: icon_name.into(),
        selected: active,
        checked: active,
        frame: TemplateNodeFrameData {
            x: 6.0,
            y: 6.0,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn resolved_panel_surface(background: [u8; 4], border: [u8; 4]) -> ResolvedButtonStyle {
    resolved_panel_surface_with_radius(background, border, 0.0)
}

fn resolved_panel_surface_with_radius(
    background: [u8; 4],
    border: [u8; 4],
    corner_radius: f32,
) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                background[0],
                background[1],
                background[2],
                background[3],
            ))),
            border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                border[0], border[1], border[2], border[3],
            ))),
            corner_radius,
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

fn frame_rect(frame: &TemplateNodeFrameData) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
