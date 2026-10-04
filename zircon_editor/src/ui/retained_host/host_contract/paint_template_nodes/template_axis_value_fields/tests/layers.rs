use super::*;

#[test]
fn axis_value_text_paints_above_field_surface() {
    let surface = 40;

    assert!(surface < value_text_order(surface));
}
