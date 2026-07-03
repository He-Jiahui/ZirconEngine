use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_are_child_owner() {
    assert_code_review_findings_status_docs_are_synced();
}

#[test]
fn runtime_15_code_review_findings_status_row_source_reads_structure_guard_children() {
    let status_rows = review_guard_status_rows_source();
    assert_contains_all(
        "review-guard status row aggregate reads structure-guard child tree",
        &status_rows,
        &[
            STATUS_ROW_SOURCE_SYNC_SLICE,
            STATUS_ROW_SOURCE_SYNC_ID,
            STATUS_ROW_SOURCE_SYNC_GUARD,
            STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
            "Runtime 15 M3 code-review structure-guard row-data folder-backed split",
            "runtime_15_code_review_structure_guard_row_data_folder_backed_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/status_docs.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/typed_error.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/row_data_owner.rs",
        ],
    );
}

pub(super) fn assert_code_review_findings_status_docs_are_synced() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = review_guard_status_rows_source();
    let status_maps = format!(
        "{}\n{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH)
    );

    source_anchors::assert_code_review_findings_status_doc_source_anchors(
        [
            ("Runtime 15 plan", runtime_15_plan.as_str()),
            ("Runtime index", runtime_index.as_str()),
            ("review findings", review_findings.as_str()),
            ("structure convention", structure_convention.as_str()),
            ("module convention doc", module_doc.as_str()),
            ("status-output row data", status_rows.as_str()),
        ],
        status_anchors::STATUS_DOC_CHILD_ANCHORS,
    );

    assert_contains_all(
        "status/date expected-slice maps",
        &status_maps,
        status_anchors::STATUS_DOC_MAP_ANCHORS,
    );
    assert_contains_all(
        "runtime architecture session note",
        &session_note,
        status_anchors::STATUS_DOC_SESSION_ANCHORS,
    );
}
