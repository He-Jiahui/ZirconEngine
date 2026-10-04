use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    chevron_rect, nested_label_rect, nested_select_field_rect, shadow_check_rect,
};
use super::super::primitives::push_text;
use super::super::push_inspector_row_commands;
use super::support::inspector_node;

#[test]
fn nested_lighting_select_preserves_right_edge_with_select_indent() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 304.0,
        height: 28.0,
    };

    let label = nested_label_rect(&rect);
    let field = nested_select_field_rect(&rect);

    assert_eq!(label.x, 22.0);
    assert_eq!(field.x, 162.0);
    assert_eq!(field.width, 150.0);
    assert_eq!(field.x + field.width, rect.x + rect.width);
}

#[test]
fn narrow_inspector_rows_keep_every_child_rect_inside_the_parent() {
    let rect = FrameRect {
        x: 5.0,
        y: 3.0,
        width: 12.0,
        height: 2.0,
    };
    let node = inspector_node("WorkbenchShadowCheck", "Cast Shadow", "true");
    let label = nested_label_rect(&rect);
    let field = nested_select_field_rect(&rect);
    let chevron = chevron_rect(&field, 10.0);
    let check = shadow_check_rect(&node, &rect);

    for child in [label, field, chevron, check] {
        assert!(child.x.is_finite() && child.y.is_finite());
        assert!(child.width.is_finite() && child.height.is_finite());
        assert!(child.x >= rect.x);
        assert!(child.y >= rect.y);
        assert!(child.x + child.width <= rect.x + rect.width);
        assert!(child.y + child.height <= rect.y + rect.height);
    }
}

#[test]
fn empty_inspector_text_slot_does_not_emit_a_paint_command() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 0.0,
        height: 16.0,
    };
    let mut commands: Vec<HostPaintCommand> = Vec::new();

    push_text(
        &mut commands,
        rect.clone(),
        &rect,
        0,
        "No available width",
        [255, 255, 255, 255],
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn unpaintable_resource_row_does_not_emit_partial_controls() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 12.0,
        height: 2.0,
    };
    let node = inspector_node("WorkbenchMeshRow", "Mesh", "Box_01");
    let mut commands: Vec<HostPaintCommand> = Vec::new();

    assert!(push_inspector_row_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands.is_empty());
}
