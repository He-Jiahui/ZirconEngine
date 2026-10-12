use super::semantic_paths::{layout_semantic_action_path, slot_semantic_action_path};

#[test]
fn layout_semantic_action_path_maps_linear_box_gap_action() {
    assert_eq!(
        layout_semantic_action_path("layout.box.gap.set"),
        Some("container.gap")
    );
}

#[test]
fn slot_semantic_action_path_maps_linear_slot_actions() {
    assert_eq!(
        slot_semantic_action_path("slot.linear.width_weight.set"),
        Some("layout.width.weight")
    );
    assert_eq!(
        slot_semantic_action_path("slot.linear.width_stretch.set"),
        Some("layout.width.stretch")
    );
    assert_eq!(
        slot_semantic_action_path("slot.linear.height_weight.set"),
        Some("layout.height.weight")
    );
    assert_eq!(
        slot_semantic_action_path("slot.linear.height_stretch.set"),
        Some("layout.height.stretch")
    );
}
