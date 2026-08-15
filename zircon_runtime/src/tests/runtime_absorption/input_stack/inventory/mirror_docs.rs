#[test]
fn runtime_12_input_stack_mirror_docs_match_structure_audit_counts() {
    let input_doc = include_str!("../../../../../../docs/zircon_runtime/input/input_state.md");
    let closeout = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md"
    );

    for required_anchor in [
        "input_stack_boundary",
        "expected_runtime_module_count = 19",
        "expected_framework_module_count = 26",
        "expected_test_module_count = 7",
        "expected_guard_file_count = 6",
        "ActionEvaluationGeneration",
        "ActionEvaluationWorkspace",
        "FrameAxisIndex",
        "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
    ] {
        assert!(
            input_doc.contains(required_anchor),
            "Runtime input module doc should mirror Runtime 12 input-stack audit anchor `{required_anchor}`"
        );
    }

    for current_anchor in [
        "expected_runtime_module_count = 19",
        "expected_framework_module_count = 26",
    ] {
        assert_eq!(
            input_doc.matches(current_anchor).count(),
            1,
            "Runtime input module doc should contain exactly one current `{current_anchor}` anchor"
        );
    }

    for summary_anchor in [
        "# Runtime12 M4 Input Event Bounds Current-Source Addendum",
        "Milestone: M4",
        "runtime/framework/test `18/26/7`",
        "behavior anchors `21`",
        "unexpected/missing/wiring/risk lists 全空",
    ] {
        assert!(
            closeout.contains(summary_anchor),
            "Runtime 12 M4 output record should retain concise acceptance anchor `{summary_anchor}`"
        );
    }

    for stale_current_claim in [
        "Current evidence reports `expected_runtime_module_count = 12`",
        "Direct audit still reports runtime/framework/test owner modules 12/20/7",
        "The direct Runtime12 structure audit reports runtime/framework/test/guard counts 12/20/7/6",
    ] {
        assert!(
            !input_doc.contains(stale_current_claim),
            "Runtime input module doc must label historical inventory evidence as historical: `{stale_current_claim}`"
        );
    }
}

#[test]
fn runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation() {
    let input_doc = include_str!("../../../../../../docs/zircon_runtime/input/input_state.md");

    assert!(
        input_doc.contains("Runtime 12 managed Cargo gates remain pending"),
        "the input module doc must not present static review as managed Cargo evidence"
    );
    for cargo_gate in [
        "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
        "cargo test -p zircon_runtime --lib action_map --locked -- --nocapture",
        "cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture",
        "cargo test -p zircon_app --locked",
    ] {
        assert!(
            input_doc.contains(cargo_gate),
            "the input module doc must retain pending managed Cargo gate `{cargo_gate}`"
        );
    }
}
