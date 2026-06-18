use super::super::super::data::TemplateNodeFrameData;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn status_control_kind_matches_workbench_status_ids() {
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusReady", "Ready", 96.0, 46.0)),
        Some(StatusControlKind::Signal(StatusSignalKind::Ready))
    );
    assert_eq!(
        status_control_kind(&status_node(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            112.0,
            30.0
        )),
        Some(StatusControlKind::Chip)
    );
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusTarget", "", 34.0, 30.0)),
        Some(StatusControlKind::Icon(StatusIconKind::Target))
    );
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusFill", "", 80.0, 46.0)),
        None
    );
}

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

#[test]
fn status_chip_paints_pill_surface_and_down_chevron() {
    let bytes = paint_template_nodes_for_test(
        140,
        48,
        model_rc(vec![status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm")]),
    );

    assert_ne!(pixel_at(&bytes, 140, 20, 20), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 140, 60, 9), STATUS_RIGHT_BORDER);
    assert!(changed_pixel_count(&bytes, 140, 101, 18, 18, 14) > 0);
}

#[test]
fn status_chip_uses_declared_text_color_and_layout_offset() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.layout_offset_y = -2.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144);

    let rect = status_control_offset_rect(
        &node,
        &FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    );

    assert!((rect.y - 7.0).abs() < 0.001);
    assert_eq!(status_chip_text_color(&node), [125, 137, 144, 255]);
}

#[test]
fn status_chip_uses_shared_painter_state_priority() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.hovered = true;
    node.selected = true;
    let hovered = select_workbench_status_chip_style(&node);
    assert_eq!(
        hovered.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Hovered
    );
    assert_eq!(hovered.background, PALETTE.surface_hover);

    node.pressed = true;
    let pressed = select_workbench_status_chip_style(&node);
    assert_eq!(
        pressed.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.disabled = true;
    let disabled = select_workbench_status_chip_style(&node);
    assert_eq!(
        disabled.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
    );
    assert_eq!(disabled.background, PALETTE.surface_disabled);
}

#[test]
fn status_icon_button_paints_target_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        42,
        model_rc(vec![status_icon_node("WorkbenchStatusTarget")]),
    );

    assert_ne!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 48, 24, 6), STATUS_RIGHT_BORDER);
    assert!(changed_pixel_count(&bytes, 48, 14, 11, 20, 20) > 0);
}

#[test]
fn status_icon_button_uses_declared_layout_offset() {
    let mut node = status_icon_node("WorkbenchStatusTarget");
    node.layout_offset_y = -2.0;

    let rect = status_control_offset_rect(
        &node,
        &FrameRect {
            x: 6.0,
            y: 6.0,
            width: 34.0,
            height: 30.0,
        },
    );

    assert!((rect.y - 4.0).abs() < 0.001);
}

#[test]
fn status_icon_button_uses_shared_icon_button_state_priority() {
    let mut node = status_icon_node("WorkbenchStatusTarget");
    node.checked = true;
    let checked = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        checked.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Checked
    );
    assert_eq!(checked.glyph, PALETTE.focus_ring);

    node.hovered = true;
    let hovered = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        hovered.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Hovered
    );

    node.pressed = true;
    let pressed = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        pressed.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed.background, PALETTE.surface_pressed);
}

fn status_node(control_id: &str, text: &str, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn status_chip_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 9.0,
            width: 112.0,
            height: 30.0,
        },
        ..status_node(control_id, text, 112.0, 30.0)
    }
}

fn status_icon_node(control_id: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "IconButton".into(),
        frame: TemplateNodeFrameData {
            x: 6.0,
            y: 6.0,
            width: 34.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
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
