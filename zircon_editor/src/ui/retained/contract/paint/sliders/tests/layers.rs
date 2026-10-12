use super::*;

#[test]
fn slider_layers_keep_track_ticks_thumbs_and_value_stack_order() {
    let order = 30;

    assert!(order < track_fill_order(order));
    assert!(track_fill_order(order) < tick_order(order));
    assert_eq!(label_order(order), range_min_thumb_order(order));
    assert!(range_min_thumb_order(order) < primary_thumb_order(order));
    assert!(primary_thumb_order(order) < value_surface_order(order));
    assert!(value_surface_order(order) < inner_text_order(value_surface_order(order)));
    assert!(range_min_thumb_order(order) < thumb_body_order(range_min_thumb_order(order)));
    assert!(primary_thumb_order(order) < thumb_body_order(primary_thumb_order(order)));
}
