use super::*;

#[test]
fn runtime_15_f12_production_dead_code_current_state_is_zero_hit() {
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
    );

    let s10_row = slice_entry(&runtime_15_plan, F12_CURRENT_STATE_WORDING_SLICE)
        .expect("Runtime 15 numbered output should keep the F12 current-state record");
    assert_contains_all(
        "Runtime 15 S10 row",
        s10_row,
        &[
            F12_CURRENT_STATE_WORDING_SLICE,
            F12_CURRENT_STATE_WORDING_STATUS,
            F12_CURRENT_STATE_WORDING_GUARD,
            CURRENT_PRODUCTION_GATE,
            CURRENT_F12_ZERO_HIT_WORDING,
            "完整 Runtime 15 Cargo sweep 仍 pending",
        ],
    );
    assert!(
        !s10_row.contains(STALE_OTHER_SUPPRESSION_SWEEP_PENDING),
        "S10 current-state row should not reopen production suppression cleanup after the global zero-hit gate"
    );
    assert!(
        !s10_row.contains(STALE_FULL_CRATE_DEAD_CODE_SWEEP_PENDING),
        "S10 current-state row should not keep the stale full-crate dead-code sweep pending wording"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                F12_CURRENT_STATE_WORDING_SLICE,
                F12_CURRENT_STATE_WORDING_STATUS,
                F12_CURRENT_STATE_WORDING_GUARD,
                CURRENT_PRODUCTION_GATE,
                CURRENT_F12_ZERO_HIT_WORDING,
            ],
        );
    }

    let f12_review_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F12 |"))
        .expect("review findings should keep the F12 top row");
    assert_contains_all(
        "F12 review row",
        f12_review_row,
        &[
            "Runtime production `allow(dead_code)` sweep is globally gated",
            CURRENT_PRODUCTION_GATE,
            CURRENT_F12_ZERO_HIT_WORDING,
        ],
    );

    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            F12_CURRENT_STATE_WORDING_SLICE,
            F12_CURRENT_STATE_WORDING_STATUS,
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &[F12_CURRENT_STATE_WORDING_SLICE, "2026-06-30"],
    );
}
