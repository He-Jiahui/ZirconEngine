pub(super) const TYPED_ERROR_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_CHILD_SOURCES_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/child_sources.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/delegation.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_DOC_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/paths.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/source_helper_ownership.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/source_helper_status.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCES_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/sources.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/status_maps.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/status_mirrors.rs";

pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs";
pub(super) const REVIEW_GUARD_STATUS_ROW_CHILD_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs",
];
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps.rs";
pub(super) const REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps.rs";

pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 typed-error structure status-doc guard folder-backed split";
pub(super) const TYPED_ERROR_STATUS_DOCS_GUARD_SPLIT_ID: &str =
    "runtime_15_typed_error_structure_status_docs_folder_backed_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_NAME: &str =
    "Runtime 15 M3 typed-error status-doc source helper child split";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_ID: &str =
    "runtime_15_typed_error_status_doc_source_helper_child_split_static_passed_cargo_deferred";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_DATE: &str = "2026-07-05";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_GUARD: &str =
    "runtime_15_typed_error_status_doc_source_helpers_are_child_backed";
pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_GUARD: &str =
    "runtime_15_typed_error_status_doc_source_helper_status_is_current";
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

pub(super) const TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_sources",
        TYPED_ERROR_STATUS_DOCS_CHILD_SOURCES_CHILD,
        "pub(super) fn typed_error_status_docs_child_source_blob",
    ),
    (
        "paths",
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD,
        "TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN",
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "source_helper_status",
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_CHILD,
        TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_STATUS_DOCS_SOURCES_CHILD,
        "pub(super) fn typed_error_status_doc_sources",
    ),
];
