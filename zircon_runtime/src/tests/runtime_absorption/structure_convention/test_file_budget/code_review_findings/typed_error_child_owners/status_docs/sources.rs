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
