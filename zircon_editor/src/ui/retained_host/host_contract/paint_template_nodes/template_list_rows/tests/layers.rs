use super::*;

#[test]
fn list_row_layers_keep_surface_indicator_label_adornment_order() {
    let surface = 12;

    assert!(surface < selection_indicator_order(surface));
    assert!(selection_indicator_order(surface) < label_order(surface));
    assert!(label_order(surface) < adornment_order(surface));
}
