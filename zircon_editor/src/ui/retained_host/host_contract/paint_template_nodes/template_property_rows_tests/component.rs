use super::super::super::template_row_metrics::workbench_row_palette;
use super::super::identity::is_property_row;
use super::super::layout::{component_property_label_width, property_label_width};
use super::super::push_property_row_text_commands;
use super::support::{component_property_node, frame};

#[test]
fn component_property_input_rows_use_split_property_row_painter() {
    let node = component_property_node();

    assert!(is_property_row(&node));
    assert_eq!(
        property_label_width(&node, &frame(0.0, 0.0, 360.0, 28.0)),
        component_property_label_width()
    );
}

#[test]
fn selected_component_property_scalar_value_paints_neutral_field_border() {
    let mut node = component_property_node();
    node.control_id = "WorkbenchComponentPropertyVirtualRowSelected".into();
    node.selected = true;
    node.value_text = "1.0".into();
    let rect = frame(0.0, 0.0, 240.0, 30.0);
    let mut commands = Vec::new();

    assert!(push_property_row_text_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));

    let palette = workbench_row_palette();
    let field = commands
        .iter()
        .find(|command| command.background_color == Some(palette.property_field_surface))
        .expect("selected scalar property should paint a value field surface");
    assert_eq!(field.border_color, Some(palette.property_field_border));
    assert_ne!(
        field.border_color,
        Some(palette.property_field_focus_border)
    );
}

#[test]
fn fully_clipped_property_row_does_not_emit_paint_commands() {
    let node = component_property_node();
    let rect = frame(8.0, 6.0, 240.0, 28.0);
    let clip = frame(280.0, 0.0, 80.0, 80.0);
    let mut commands = Vec::new();

    assert!(push_property_row_text_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_property_row_keeps_only_clipped_paint_commands() {
    let node = component_property_node();
    let rect = frame(8.0, 6.0, 240.0, 28.0);
    let clip = frame(16.0, 8.0, 60.0, 20.0);
    let mut commands = Vec::new();

    assert!(push_property_row_text_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands.iter().all(|command| {
        command
            .clip_frame
            .as_ref()
            .is_some_and(|clip_frame| frame_is_within(&clip, clip_frame))
    }));
}

fn frame_is_within(
    outer: &crate::ui::retained_host::host_contract::data::FrameRect,
    inner: &crate::ui::retained_host::host_contract::data::FrameRect,
) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
