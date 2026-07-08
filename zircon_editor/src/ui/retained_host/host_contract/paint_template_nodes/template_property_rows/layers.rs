const VALUE_GROUP_OFFSET: i32 = 1;
const FIELD_TEXT_OFFSET: i32 = 1;

pub(super) fn value_group_order(label_order: i32) -> i32 {
    label_order + VALUE_GROUP_OFFSET
}

pub(super) fn field_text_order(field_surface_order: i32) -> i32 {
    field_surface_order + FIELD_TEXT_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_row_orders_keep_values_above_labels_and_field_text_above_surface() {
        let label = 30;
        let value_group = value_group_order(label);

        assert!(label < value_group);
        assert!(value_group < field_text_order(value_group));
    }
}
