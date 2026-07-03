use super::super::*;

#[path = "p0_native_fixture_leaf_owners/budgets.rs"]
mod budgets;
#[path = "p0_native_fixture_leaf_owners/delegation.rs"]
mod delegation;
#[path = "p0_native_fixture_leaf_owners/leaf_ownership.rs"]
mod leaf_ownership;
#[path = "p0_native_fixture_leaf_owners/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners.rs";
pub(super) const SLICE: &str = "Runtime 15 M3 P0 native fixture review guard leaf-owner split";
pub(super) const STATUS: &str =
    "runtime_15_p0_native_fixture_review_guard_leaf_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str = "runtime_15_p0_native_fixture_review_guards_are_leaf_owners";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str =
    "runtime_15_p0_native_fixture_leaf_owner_guard_budgets_are_focused";
pub(super) const PARENT: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs";
pub(super) const SDK_MACRO_LEAF: &str = "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs";
pub(super) const IMPORTER_LEAF: &str = "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs";
pub(super) const SDK_MACRO_REVIEW: &str =
    "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest";
pub(super) const IMPORTER_REVIEW: &str = "review_d13_native_fixture_importer_is_manifest_described";
pub(super) const REVIEW_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/delegation.rs",
        FOLDER_BACKED_GUARD,
    ),
    (
        "leaf_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/leaf_ownership.rs",
        GUARD,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/status_mirrors.rs",
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/budgets.rs",
        BUDGET_GUARD,
    ),
];

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut source = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        source.push_str(&child_source);
        source.push('\n');
    }
    source
}
