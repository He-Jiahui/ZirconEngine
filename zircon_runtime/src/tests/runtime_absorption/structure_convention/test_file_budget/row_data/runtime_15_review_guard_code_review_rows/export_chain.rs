use super::*;

#[test]
fn runtime_15_review_guard_code_review_row_exports_are_current() {
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_review_guard_exports = read_runtime_src(RUNTIME_15_REVIEW_GUARD_EXPORTS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3_review_guard_exports =
        read_runtime_src(RUNTIME_15_M3_REVIEW_GUARD_EXPORTS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);

    assert_contains_prefixed_exports(
        "review-guard row-data parent exports every code-review child group",
        &review_guard_splits,
        "CODE_REVIEW_",
    );
    assert_contains_all(
        "Runtime 15 M3 re-exports review-guard child groups",
        &runtime_15_m3,
        &["pub(super) use review_guard_exports::*;"],
    );
    assert_contains_prefixed_exports(
        "Runtime 15 M3 review-guard exports expose every code-review child group",
        &runtime_15_m3_review_guard_exports,
        "REVIEW_GUARD_CODE_REVIEW_",
    );
    assert_contains_all(
        "Runtime 15 row-data parent exports every code-review child group",
        &runtime_15,
        &["pub(super) use m3_review_guard_exports::*;"],
    );
    assert_contains_prefixed_exports(
        "Runtime 15 review-guard exports expose every code-review child group",
        &runtime_15_review_guard_exports,
        "RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_",
    );
    assert_contains_prefixed_exports(
        "top-level expected status row data consumes every code-review child group",
        &top_level,
        "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_",
    );

    let status_rows = status_support_review_guard_source_blob();
    let status_map = status_support_status_map_source_blob();
    let date_map = status_support_date_map_source_blob();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let status_anchors = [
        EXPORT_STATUS_SOURCE_RECONCILIATION_STATUS_NAME,
        EXPORT_STATUS_SOURCE_RECONCILIATION_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/export_chain.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_exports.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3_review_guard_exports.rs",
        EXPORT_STATUS_SOURCE_RECONCILIATION_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard review rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status-support map records code-review export/status source reconciliation",
        &status_map,
        &[
            EXPORT_STATUS_SOURCE_RECONCILIATION_STATUS_NAME,
            EXPORT_STATUS_SOURCE_RECONCILIATION_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records code-review export/status source reconciliation",
        &date_map,
        &[
            EXPORT_STATUS_SOURCE_RECONCILIATION_STATUS_NAME,
            "2026-07-07",
        ],
    );
}

fn assert_contains_prefixed_exports(label: &str, source: &str, prefix: &str) {
    for suffix in CODE_REVIEW_CHILD_EXPORT_SUFFIXES {
        let export_name = format!("{prefix}{suffix}");
        assert!(
            source.contains(&export_name),
            "{label} should contain {export_name}"
        );
    }
}
