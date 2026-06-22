use super::super::template_node_label;
use super::support::property_row_node;

#[test]
fn property_row_label_keeps_label_and_value_visible() {
    assert_eq!(
        template_node_label(&property_row_node(), None),
        "Position    X 12.0   Y 3.5   Z -8.0"
    );
}
