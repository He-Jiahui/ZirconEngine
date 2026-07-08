use super::super::{icon_button_context, icon_button_paint_rect, icon_glyph_rect};
use super::support::{frame_rect, icon_node};

#[test]
fn toolbar_icon_button_uses_unreal_slim_toolbar_icon_size() {
    let node = icon_node(
        "WorkbenchToolbarMenu",
        "zircon_editor_shell/toolbar/menu.svg",
        false,
        30.0,
        30.0,
    );

    let paint_rect = icon_button_paint_rect(&node, &frame_rect(&node.frame));
    let glyph = icon_glyph_rect(&node, &paint_rect, icon_button_context(&node));

    assert!((glyph.x - 11.0).abs() < 0.001);
    assert!((glyph.y - 11.0).abs() < 0.001);
    assert_eq!(glyph.width, 20.0);
    assert_eq!(glyph.height, 20.0);
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
fn panel_icon_button_defaults_to_unreal_icon16_size() {
    let node = icon_node(
        "WorkbenchMiniAdd",
        "zircon_editor_shell/controls/add.svg",
        false,
        38.0,
        38.0,
    );

    let paint_rect = icon_button_paint_rect(&node, &frame_rect(&node.frame));
    let glyph = icon_glyph_rect(&node, &paint_rect, icon_button_context(&node));

    assert!((glyph.x - 17.0).abs() < 0.001);
    assert!((glyph.y - 17.0).abs() < 0.001);
    assert_eq!(glyph.width, 16.0);
    assert_eq!(glyph.height, 16.0);
}

#[test]
fn rail_icon_button_defaults_to_unreal_large_icon24_size() {
    let node = icon_node(
        "WorkbenchRailAssets",
        "zircon_editor_shell/rail/assets.svg",
        false,
        48.0,
        48.0,
    );

    let paint_rect = icon_button_paint_rect(&node, &frame_rect(&node.frame));
    let glyph = icon_glyph_rect(&node, &paint_rect, icon_button_context(&node));

    assert!((glyph.x - 18.0).abs() < 0.001);
    assert!((glyph.y - 18.0).abs() < 0.001);
    assert_eq!(glyph.width, 24.0);
    assert_eq!(glyph.height, 24.0);
}
