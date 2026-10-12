use super::*;

#[test]
fn dropdown_layers_keep_surface_label_chevron_order() {
    let surface = 40;

    assert!(surface < label_order(surface));
    assert!(label_order(surface) < chevron_order(surface));
}
