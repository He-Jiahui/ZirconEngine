use super::super::super::super::*;
use super::*;

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
            "{}\n{}",
            typed_error_status_map_source(),
            typed_error_date_map_source()
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
    for path in REVIEW_GUARD_TYPED_ERROR_ROW_PATHS
        .iter()
        .chain(REVIEW_GUARD_STATUS_SUPPORT_ROW_PATHS.iter())
    {
        status_rows.push('\n');
        status_rows.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_ROW_PATHS {
        status_rows.push('\n');
        status_rows.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_STRUCTURE_ASSERTION_ROW_PATHS {
        status_rows.push('\n');
        status_rows.push_str(&read_runtime_src(path));
    }
    status_rows
}

pub(super) fn typed_error_status_doc_row_child_sources() -> Vec<(&'static str, String)> {
    REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_ROW_PATHS
        .iter()
        .chain(REVIEW_GUARD_TYPED_ERROR_STRUCTURE_ASSERTION_ROW_PATHS.iter())
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_status_map_source() -> String {
    let mut status_maps = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    for path in REVIEW_GUARD_STATUS_MAP_PATHS {
        status_maps.push('\n');
        status_maps.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_CHILD_PATHS {
        status_maps.push('\n');
        status_maps.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_STATUS_MAP_PATHS {
        status_maps.push('\n');
        status_maps.push_str(&read_runtime_src(path));
    }
    status_maps
}

pub(super) fn typed_error_date_map_source() -> String {
    let mut date_maps = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );
    for path in REVIEW_GUARD_DATE_MAP_PATHS {
        date_maps.push('\n');
        date_maps.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_DATE_MAP_CHILD_PATHS {
        date_maps.push('\n');
        date_maps.push_str(&read_runtime_src(path));
    }
    for path in REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_DATE_MAP_PATHS {
        date_maps.push('\n');
        date_maps.push_str(&read_runtime_src(path));
    }
    date_maps
}

const REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_ROW_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/foundation.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/delegation.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/delegation/core.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/delegation/sources.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/delegation/split_layout.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/paths.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/paths/core.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/paths/child_inventory.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/paths/status_current.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/status_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/maps/core.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/maps/sources.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/maps/split_layout.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/status_mirrors.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/mirrors/core.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/mirrors/sources.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/mirrors/split_layout.rs",
];

const REVIEW_GUARD_TYPED_ERROR_STRUCTURE_ASSERTION_ROW_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/child_ownership.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure/foundation.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure/convergence_mounts.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure/moved_guard_absence.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure/native_plugin_loader.rs",
];

const REVIEW_GUARD_TYPED_ERROR_ROW_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/native_plugin_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/runtime_surface_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/asset_shader_rows.rs",
];

const REVIEW_GUARD_STATUS_SUPPORT_ROW_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/typed_error_status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_foundation_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_delegation_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_inventory_metadata_rows.rs",
];

const REVIEW_GUARD_STATUS_MAP_PATHS: &[&str] = &[
    REVIEW_GUARD_STATUS_MAP_PATH,
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/foundation_review_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/plugin_importer_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/top_row_review_maps.rs",
];

const REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_CHILD_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/review_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/row_data_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/source_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/moved_guard_absence_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/native_plugin_loader_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/structure_assertion_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/structure_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/top_level_maps.rs",
];

const REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_STATUS_MAP_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/base_status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/delegation_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/paths_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/status_maps_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/status_mirrors_rows.rs",
];

const REVIEW_GUARD_DATE_MAP_PATHS: &[&str] = &[
    REVIEW_GUARD_DATE_MAP_PATH,
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/foundation_review_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/plugin_importer_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/top_row_review_maps.rs",
];

const REVIEW_GUARD_TYPED_ERROR_DATE_MAP_CHILD_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/expected_slice_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/review_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/row_data_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/source_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/moved_guard_absence_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/native_plugin_loader_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/structure_assertion_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/structure_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/top_level_maps.rs",
];

const REVIEW_GUARD_TYPED_ERROR_STATUS_DOC_DATE_MAP_PATHS: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/base_status_doc_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/delegation_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/paths_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/status_maps_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/status_mirrors_rows.rs",
];
