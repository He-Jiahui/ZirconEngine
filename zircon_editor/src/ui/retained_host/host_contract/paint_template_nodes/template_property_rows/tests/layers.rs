use super::*;

#[test]
fn property_row_orders_keep_values_above_labels_and_field_text_above_surface() {
    let label = 30;
    let value_group = value_group_order(label);

    assert!(label < value_group);
    assert!(value_group < field_text_order(value_group));
}
