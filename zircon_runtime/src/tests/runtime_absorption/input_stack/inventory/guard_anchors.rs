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
    let cargo_gate_guard = include_str!("../../plan_status/cargo_gates/late/runtime_12.rs");
    for guard_anchor in [
        "runtime_12_input_stack_contracts_stay_documented_and_exported",
        "runtime_12_action_mapping_keeps_ui_filtered_evaluation_path",
        "runtime_12_gamepad_bridge_keeps_runtime_abi_path",
        "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
        "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
    ] {
        assert!(
            input_stack_guard_sources
                .iter()
                .any(|source| source.contains(guard_anchor))
                || cargo_gate_guard.contains(guard_anchor),
            "Runtime 12 guard anchor `{guard_anchor}` should stay visible to input_stack_boundary"
        );
    }
}
