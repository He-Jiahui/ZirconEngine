use super::super::super::data::TemplateNodeFrameData;
use super::super::style_selector::{
    select_workbench_tooltip_style, WORKBENCH_TOOLTIP_BODY, WORKBENCH_TOOLTIP_BORDER,
    WORKBENCH_TOOLTIP_ICON, WORKBENCH_TOOLTIP_SURFACE,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn workbench_tooltip_paints_declared_bubble_arrow_and_info_icon() {
    let mut node = tooltip_node();
    node.value_number = 8.0;
    node.value_color = Color::from_rgb_u8(23, 28, 32);
    node.label_color = Color::from_rgb_u8(168, 179, 184);
    node.icon_color = Color::from_rgb_u8(37, 156, 167);

    let style = select_workbench_tooltip_style(&node);
    assert_eq!(tooltip_arrow_size(&node), 8.0);
    assert_eq!(style.arrow, WORKBENCH_TOOLTIP_SURFACE);
    assert_eq!(style.body, WORKBENCH_TOOLTIP_BODY);
    assert_eq!(style.icon, WORKBENCH_TOOLTIP_ICON);

    let bytes = paint_template_nodes_for_test(128, 96, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 128, 64, 12), WORKBENCH_TOOLTIP_SURFACE);
    assert_eq!(pixel_at(&bytes, 128, 64, 8), WORKBENCH_TOOLTIP_BORDER);
    assert_eq!(pixel_at(&bytes, 128, 63, 56), WORKBENCH_TOOLTIP_SURFACE);
    assert_eq!(pixel_at(&bytes, 128, 63, 69), WORKBENCH_TOOLTIP_ICON);
    assert!(changed_pixel_count(&bytes, 128, 22, 14, 50, 14) > 0);
    assert!(changed_pixel_count(&bytes, 128, 22, 29, 72, 14) > 0);
}

#[test]
fn workbench_tooltip_style_uses_shared_state_priority() {
    let mut node = tooltip_node();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_tooltip_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_ne!(disabled.border, WORKBENCH_TOOLTIP_BORDER);

    node.disabled = false;
    let pressed = select_workbench_tooltip_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);

    node.pressed = false;
    let focused = select_workbench_tooltip_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
}

fn tooltip_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchTooltipRoot".into(),
        role: "Tooltip".into(),
        component_role: "tooltip".into(),
        surface_variant: "workbench-tooltip".into(),
        text: "Tooltip".into(),
        label_text: "This is a tooltip".into(),
        layout_icon_size: 18.0,
        layout_content_offset_y: 56.0,
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 110.0,
            height: 78.0,
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
    for row in y..(y + height) {
        for column in x..(x + width) {
            if pixel_at(bytes, frame_width, column, row) != [0, 0, 0, 255] {
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
