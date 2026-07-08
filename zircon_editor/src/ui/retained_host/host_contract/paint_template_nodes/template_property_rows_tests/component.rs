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
