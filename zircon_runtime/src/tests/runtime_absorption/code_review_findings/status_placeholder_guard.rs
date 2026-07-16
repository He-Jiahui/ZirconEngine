const REVIEW_OUTPUT: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
);

#[test]
fn review_numbered_output_uses_concrete_status_anchors_instead_of_placeholders() {
    let placeholder = ["对应子计划", "状态锚"].concat();
    assert!(
        !REVIEW_OUTPUT.contains(&placeholder),
        "numbered review output should not preserve placeholder status anchors"
    );
    assert!(
        REVIEW_OUTPUT
            .contains("engine_review_findings_current_evidence_rows_reconciled_static_passed"),
        "numbered review output should record the concrete row-reconciliation status"
    );
}
