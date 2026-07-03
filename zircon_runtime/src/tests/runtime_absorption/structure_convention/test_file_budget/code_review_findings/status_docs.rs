use super::super::*;

#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/source_anchor_guard.rs"]
mod source_anchor_guard;
#[path = "status_docs/source_anchors.rs"]
mod source_anchors;
#[path = "status_docs/status_anchor_guard.rs"]
mod status_anchor_guard;
#[path = "status_docs/status_anchors.rs"]
mod status_anchors;
#[path = "status_docs/sync.rs"]
mod sync;

pub(super) const STATUS_DOC_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs";
pub(super) const STATUS_DOC_SOURCE_ANCHORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors.rs";
pub(super) const STATUS_DOC_STATUS_ANCHORS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors.rs";
pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

const REVIEW_GUARD_STATUS_ROW_SOURCE_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs",
    REVIEW_GUARD_STATUS_ROWS_PATH,
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/status_docs.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/typed_error.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/row_data_owner.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
];

pub(super) const STATUS_DOC_SOURCE_ANCHORS_SLICE: &str =
    "Runtime 15 M3 code review findings status-doc source anchors child-owner split";
pub(super) const STATUS_DOC_SOURCE_ANCHORS_STATUS: &str =
    "runtime_15_code_review_findings_status_docs_source_anchors_child_owner_split_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_SOURCE_ANCHORS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings status-doc source anchors folder-backed split";
pub(super) const STATUS_DOC_SOURCE_ANCHORS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_static_passed_cargo_deferred";
pub(super) const STATUS_DOC_SOURCE_ANCHORS_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner";
pub(super) const STATUS_DOC_FOLDER_BACKED_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings status-doc guard folder-backed split";
pub(super) const STATUS_DOC_FOLDER_BACKED_SPLIT_ID: &str =
    "runtime_15_code_review_findings_status_docs_folder_backed_static_passed_cargo_deferred";
pub(super) const STATUS_ROW_SOURCE_SYNC_SLICE: &str =
    "Runtime 15 M3 code review findings status-row source child-tree sync";
pub(super) const STATUS_ROW_SOURCE_SYNC_ID: &str =
    "runtime_15_code_review_findings_status_row_source_child_tree_sync_static_passed_cargo_deferred";
pub(super) const STATUS_ROW_SOURCE_SYNC_GUARD: &str =
    "runtime_15_code_review_findings_status_row_source_reads_structure_guard_children";

pub(super) const STATUS_DOC_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "sync",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/sync.rs",
        "runtime_15_code_review_findings_status_docs_are_child_owner",
    ),
    (
        "source_anchor_guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchor_guard.rs",
        "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
    ),
    (
        "status_anchor_guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
        "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/delegation.rs",
        "runtime_15_code_review_findings_status_docs_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_code_review_findings_status_docs_are_synced() {
    sync::assert_code_review_findings_status_docs_are_synced();
}

pub(super) fn review_guard_status_rows_source() -> String {
    let mut source = String::new();
    for path in REVIEW_GUARD_STATUS_ROW_SOURCE_PATHS {
        source.push_str(&read_runtime_src(path));
        source.push('\n');
    }
    source
}

pub(super) fn status_doc_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
