use super::*;
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::{
    ICON_TINT, ICON_TINT_ACTIVE,
};

fn icon_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        icon_name: "zircon_editor_shell/toolbar/compile.svg".into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn focused_icon_image_does_not_use_active_tint() {
    let node = TemplatePaneNodeData {
        focused: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT));
}

#[test]
fn checked_icon_image_uses_active_tint_without_focus() {
    let node = TemplatePaneNodeData {
        checked: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
}

#[test]
fn popup_open_icon_image_uses_active_tint_without_focus() {
    let node = TemplatePaneNodeData {
        popup_open: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
}

#[test]
fn unknown_image_aspect_uses_the_visible_container_for_materialization() {
    let node = TemplatePaneNodeData {
        role: "Image".into(),
        media_source: "ui/editor/showcase_checker.svg".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 120.0,
        height: 80.0,
    };

    let materialization_rect = image_materialization_rect(&node, &rect, 0, 0);

    assert_eq!(materialization_rect.x, rect.x);
    assert_eq!(materialization_rect.y, rect.y);
    assert_eq!(materialization_rect.width, rect.width);
    assert_eq!(materialization_rect.height, rect.height);
}

#[test]
fn vector_raster_bucket_does_not_change_the_command_frame() {
    let node = TemplatePaneNodeData {
        role: "SvgIcon".into(),
        icon_name: "folder-open-outline".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 54.0,
        height: 54.0,
    };
    let expected_frame = image_materialization_rect(&node, &rect, 0, 0);
    let mut commands = Vec::new();

    push_template_image_command(&mut commands, &node, &rect, &rect, 7, 1.0);

    assert_eq!(commands.len(), 1);
    let command = &commands[0];
    assert_eq!(command.frame.x, expected_frame.x);
    assert_eq!(command.frame.y, expected_frame.y);
    assert_eq!(command.frame.width, expected_frame.width);
    assert_eq!(command.frame.height, expected_frame.height);
    let pixels = command
        .image_pixels
        .as_ref()
        .expect("SVG icon command should carry cached raster pixels");
    assert_eq!((pixels.width, pixels.height), (48, 48));
    assert_ne!(pixels.width as f32, command.frame.width);
}
