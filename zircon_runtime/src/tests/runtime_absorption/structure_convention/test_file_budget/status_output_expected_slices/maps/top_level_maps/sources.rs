use super::*;

pub(super) struct TopLevelMapSources {
    pub(super) status_parent: String,
    pub(super) status_runtime_15: String,
    pub(super) status_runtime_15_foundation: String,
    pub(super) status_runtime_15_naming_boundary: String,
    pub(super) status_runtime_15_m4_surface_cleanup: String,
    pub(super) status_runtime_15_m3_structure_support: String,
    pub(super) status_pre_runtime_15: String,
    pub(super) status_pre_runtime_15_runtime_01_05: String,
    pub(super) status_pre_runtime_15_runtime_06_10: String,
    pub(super) status_pre_runtime_15_runtime_11_14: String,
    pub(super) date_parent: String,
    pub(super) date_runtime_15: String,
    pub(super) date_runtime_15_foundation: String,
    pub(super) date_runtime_15_naming_boundary: String,
    pub(super) date_runtime_15_m4_surface_cleanup: String,
    pub(super) date_runtime_15_m3_structure_support: String,
    pub(super) date_pre_runtime_15: String,
    pub(super) date_pre_runtime_15_runtime_01_05: String,
    pub(super) date_pre_runtime_15_runtime_06_10: String,
    pub(super) date_pre_runtime_15_runtime_11_14: String,
    pub(super) test_budget_parent: String,
    pub(super) status_output_expected_slices_guard: String,
    pub(super) runtime_15_plan: String,
    pub(super) runtime_index: String,
    pub(super) review_findings: String,
    pub(super) structure_convention: String,
    pub(super) module_doc: String,
    pub(super) status_rows: String,
}

pub(super) fn read_top_level_map_sources() -> TopLevelMapSources {
    TopLevelMapSources {
        status_parent: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs",
        ),
        status_runtime_15: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        status_runtime_15_foundation: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
        ),
        status_runtime_15_naming_boundary: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        status_runtime_15_m4_surface_cleanup: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
        ),
        status_runtime_15_m3_structure_support: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
        ),
        status_pre_runtime_15: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
        ),
        status_pre_runtime_15_runtime_01_05: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
        ),
        status_pre_runtime_15_runtime_06_10: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
        ),
        status_pre_runtime_15_runtime_11_14: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
        ),
        date_parent: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs",
        ),
        date_runtime_15: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        date_runtime_15_foundation: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
        ),
        date_runtime_15_naming_boundary: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        date_runtime_15_m4_surface_cleanup: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
        ),
        date_runtime_15_m3_structure_support: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
        ),
        date_pre_runtime_15: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
        ),
        date_pre_runtime_15_runtime_01_05: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
        ),
        date_pre_runtime_15_runtime_06_10: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
        ),
        date_pre_runtime_15_runtime_11_14: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
        ),
        test_budget_parent: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/mod.rs",
        ),
        status_output_expected_slices_guard: read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
        ),
        runtime_15_plan: read_repo(
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        runtime_index: read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        review_findings: read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        structure_convention: read_repo("docs/plans/engine-code-structure-convention.md"),
        module_doc: read_repo("docs/zircon_runtime/structure/module-convention.md"),
        status_rows: read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
        ),
    }
}
