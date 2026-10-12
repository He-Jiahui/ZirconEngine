use super::*;

#[test]
fn tree_row_orders_keep_content_above_surface() {
    let surface = 20;
    let action_slot = action_slot_order(surface);

    assert!(surface < indent_guides_order(surface));
    assert!(indent_guides_order(surface) < disclosure_order(surface));
    assert!(disclosure_order(surface) < object_icon_order(surface));
    assert!(object_icon_order(surface) < label_order(surface));
    assert!(label_order(surface) < action_slot);
    assert!(action_slot < primary_action_icon_order(action_slot));
    assert!(primary_action_icon_order(action_slot) < secondary_action_slot_order(action_slot));
    assert!(secondary_action_slot_order(action_slot) < secondary_action_icon_order(action_slot));
}
