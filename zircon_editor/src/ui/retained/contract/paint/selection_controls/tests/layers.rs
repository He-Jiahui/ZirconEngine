use super::*;

#[test]
fn selection_control_layers_keep_surface_content_label_thumb_order() {
    let order = 40;

    assert!(order < mark_content_order(order));
    assert!(mark_content_order(order) < mark_label_order(order));
    assert!(order < toggle_label_order(order));
    assert!(toggle_label_order(order) < toggle_thumb_order(order));
}
