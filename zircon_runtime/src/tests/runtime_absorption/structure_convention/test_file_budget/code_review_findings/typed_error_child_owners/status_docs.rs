use super::super::super::*;

#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/doc_mirrors.rs"]
mod doc_mirrors;
#[path = "status_docs/status_maps.rs"]
mod status_maps;
#[path = "status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) const TYPED_ERROR_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/delegation.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_DOC_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/status_maps.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/status_mirrors.rs";
pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs";
const REVIEW_GUARD_STATUS_ROW_CHILD_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs",
];
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 typed-error structure status-doc guard folder-backed split";
pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_SPLIT_ID: &str =
    "runtime_15_typed_error_structure_status_docs_folder_backed_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD,
        "runtime_15_typed_error_status_docs_are_folder_backed",
    ),
    (
        "doc_mirrors",
        TYPED_ERROR_STATUS_DOCS_DOC_MIRRORS_CHILD,
        "assert_typed_error_status_doc_mirrors_are_synced",
    ),
    (
        "status_maps",
        TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD,
        "assert_typed_error_status_maps_are_synced",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD,
        "runtime_15_typed_error_status_docs_folder_backed_status_is_current",
    ),
];

pub(super) struct TypedErrorStatusDocSources {
    pub(super) runtime_15_plan: String,
    pub(super) runtime_index: String,
    pub(super) review_findings: String,
    pub(super) structure_convention: String,
    pub(super) module_doc: String,
    pub(super) status_rows: String,
    pub(super) status_maps: String,
}

pub(super) fn typed_error_status_doc_sources() -> TypedErrorStatusDocSources {
    TypedErrorStatusDocSources {
        runtime_15_plan: read_repo(
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        runtime_index: read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        review_findings: read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        structure_convention: read_repo("docs/plans/engine-code-structure-convention.md"),
        module_doc: read_repo("docs/zircon_runtime/structure/module-convention.md"),
        status_rows: typed_error_status_row_source(),
        status_maps: format!(
            "{}\n{}\n{}\n{}",
            read_runtime_src(
                "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            ),
            read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
            read_runtime_src(
                "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
            ),
            read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH)
        ),
    }
}

pub(super) fn typed_error_status_row_source() -> String {
    let mut status_rows = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH),
    );
    for path in REVIEW_GUARD_STATUS_ROW_CHILD_PATHS {
        status_rows.push('\n');
        status_rows.push_str(&read_runtime_src(path));
    }
    for path in [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
    ] {
        status_rows.push('\n');
        status_rows.push_str(&read_runtime_src(path));
    }
    status_rows
}

pub(super) fn assert_typed_error_status_docs_are_synced() {
    let sources = typed_error_status_doc_sources();

    doc_mirrors::assert_typed_error_status_doc_mirrors_are_synced(&sources);
    status_maps::assert_typed_error_status_maps_are_synced(&sources);
}

pub(super) fn typed_error_status_docs_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_docs_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_status_docs_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
