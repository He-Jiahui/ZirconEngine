#[test]
fn runtime_12_input_stack_mirror_docs_match_structure_audit_counts() {
    let mirror_docs = [
        (
            "Runtime input module doc",
            include_str!("../../../../../../docs/zircon_runtime/input/input_state.md"),
        ),
        (
            "Runtime 12 plan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/12/2026-07-09-input-stack-and-action-mapping-output-records.md"
            ),
        ),
        (
            "runtime index",
            include_str!(
                "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
            ),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "input_stack_boundary",
            "expected_runtime_module_count = 12",
            "expected_framework_module_count = 20",
            "expected_test_module_count = 7",
            "expected_guard_file_count = 6",
            "missing_guard_files = []",
            "public_surface_anchors = 26/26",
            "runtime_12_guard_anchors = 5/5",
            "missing_gamepad_abi_anchors = []",
            "missing_cursor_host_request_anchors = []",
            "missing_doc_anchors = []",
            "missing_test_anchors = []",
            "behavior_test_anchor_count = 15",
            "missing_behavior_test_anchors = []",
            "missing_cargo_gate_anchors = []",
            "oversized_modules = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 12 input-stack audit anchor `{required_anchor}`"
            );
        }
    }
}
