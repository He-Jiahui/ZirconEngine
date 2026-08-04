#[test]
fn runtime_12_input_stack_guard_anchors_remain_visible() {
    let input_stack_guard_sources = [
        include_str!("../../input_stack.rs"),
        include_str!("../inventory.rs"),
        include_str!("../contracts.rs"),
        include_str!("../action_mapping.rs"),
        include_str!("../gamepad_bridge.rs"),
        include_str!("mirror_docs.rs"),
    ];
}
