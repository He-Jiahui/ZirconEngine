use super::super::identity::is_property_row;
use super::super::layout::{property_label_width, COMPONENT_PROPERTY_LABEL_WIDTH};
use super::support::{component_property_node, frame};

#[test]
fn component_property_input_rows_use_split_property_row_painter() {
    let node = component_property_node();

    assert!(is_property_row(&node));
    assert_eq!(
        property_label_width(&node, &frame(0.0, 0.0, 360.0, 28.0)),
        COMPONENT_PROPERTY_LABEL_WIDTH
    );
}
