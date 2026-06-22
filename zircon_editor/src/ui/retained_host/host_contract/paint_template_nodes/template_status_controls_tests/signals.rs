use super::super::super::super::data::FrameRect;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::{
    status_signal_icon_fill, status_signal_icon_paint_rect, status_signal_icon_rect,
    status_signal_mark_color, status_signal_mark_width, status_signal_text_color,
    status_signal_text_gap, warning_mark_segments, StatusSignalKind, PALETTE,
    STATUS_NO_ERRORS_FILL,
};
use super::support::{changed_pixel_count, pixel_at, status_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn ready_status_item_paints_dot_and_text_without_chip_surface() {
    let bytes = paint_template_nodes_for_test(
        140,
        46,
        model_rc(vec![status_node(
            "WorkbenchStatusReady",
            "Ready",
            96.0,
            46.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 140, 29, 23), PALETTE.success);
    assert_eq!(pixel_at(&bytes, 140, 90, 4), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 140, 42, 14, 40, 18) > 0);
}

#[test]
fn ready_status_item_uses_declared_dot_text_and_gap_style() {
    let mut node = status_node("WorkbenchStatusReady", "Ready", 96.0, 46.0);
    node.layout_offset_x = 4.0;
    node.layout_offset_y = -1.0;
    node.layout_content_offset_x = 8.0;
    node.value_number = 9.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(78, 170, 95);

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 46.0,
        },
        StatusSignalKind::Ready,
    );

    assert!((icon.x - 28.0).abs() < 0.001);
    assert!((icon.y - 17.5).abs() < 0.001);
    assert!((icon.width - 9.0).abs() < 0.001);
    assert!((status_signal_text_gap(&node) - 8.0).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Ready),
        [143, 154, 160, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Ready),
        [78, 170, 95, 255]
    );
}

#[test]
fn errors_status_item_uses_audited_success_icon_fill() {
    let bytes = paint_template_nodes_for_test(
        140,
        46,
        model_rc(vec![status_node(
            "WorkbenchStatusErrors",
            "No Errors",
            116.0,
            46.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 140, 31, 23), STATUS_NO_ERRORS_FILL);
    assert!(changed_pixel_count(&bytes, 140, 46, 14, 58, 18) > 0);
}

#[test]
fn errors_status_item_uses_declared_success_mark_color() {
    let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 32, 24);

    assert_eq!(status_signal_mark_color(&node), [17, 32, 24, 255]);
}

#[test]
fn errors_status_item_uses_declared_visual_icon_size_without_moving_text_slot() {
    let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
    node.layout_icon_size = 12.04;

    let layout = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 116.0,
            height: 46.0,
        },
        StatusSignalKind::Success,
    );
    let paint = status_signal_icon_paint_rect(&node, &layout, StatusSignalKind::Success);

    assert!((layout.x - 24.0).abs() < 0.001);
    assert!((layout.width - 14.0).abs() < 0.001);
    assert!((paint.x - 24.98).abs() < 0.001);
    assert!((paint.width - 12.04).abs() < 0.001);
}

#[test]
fn warning_status_item_uses_declared_icon_text_and_gap_style() {
    let mut node = status_node("WorkbenchStatusWarnings", "2 Warnings", 120.0, 46.0);
    node.layout_offset_x = 5.5;
    node.layout_offset_y = -2.0;
    node.layout_content_offset_x = 6.45;
    node.layout_content_offset_y = -2.0;
    node.value_number = 21.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(135, 146, 153);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(242, 195, 86);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 24, 26);
    node.icon_stroke_width = 1.45;

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 46.0,
        },
        StatusSignalKind::Warning,
    );

    assert!((icon.x - 29.5).abs() < 0.001);
    assert!((icon.y - 8.5).abs() < 0.001);
    assert!((icon.width - 21.0).abs() < 0.001);
    assert!((status_signal_text_gap(&node) - 6.45).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Warning),
        [135, 146, 153, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Warning),
        [242, 195, 86, 255]
    );
    assert_eq!(status_signal_mark_color(&node), [17, 24, 26, 255]);
    assert!((status_signal_mark_width(&node) - 1.45).abs() < 0.001);
    let mark_segments = warning_mark_segments(&icon, status_signal_mark_width(&node));
    assert!((mark_segments[0].x - 38.9125).abs() < 0.001);
    assert!((mark_segments[0].width - 2.175).abs() < 0.001);
    assert!((mark_segments[1].height - 2.175).abs() < 0.001);
}

#[test]
fn messages_status_item_uses_declared_icon_text_and_offset_style() {
    let mut node = status_node("WorkbenchStatusMessages", "0 Messages", 130.0, 46.0);
    node.layout_offset_x = -6.0;
    node.layout_offset_y = -2.0;
    node.layout_content_offset_y = 2.0;
    node.value_number = 18.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(151, 163, 169);
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(76, 154, 232);

    let icon = status_signal_icon_rect(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 130.0,
            height: 46.0,
        },
        StatusSignalKind::Info,
    );

    assert!((icon.x - 18.0).abs() < 0.001);
    assert!((icon.y - 14.0).abs() < 0.001);
    assert!((icon.width - 18.0).abs() < 0.001);
    assert_eq!(
        status_signal_text_color(&node, StatusSignalKind::Info),
        [151, 163, 169, 255]
    );
    assert_eq!(
        status_signal_icon_fill(&node, StatusSignalKind::Info),
        [76, 154, 232, 255]
    );
}
