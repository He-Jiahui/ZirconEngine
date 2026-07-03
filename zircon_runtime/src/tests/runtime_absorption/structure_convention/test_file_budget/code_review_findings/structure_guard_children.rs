use super::super::*;

#[path = "structure_guard_children/budgets.rs"]
mod budgets;
#[path = "structure_guard_children/delegation.rs"]
mod delegation;
#[path = "structure_guard_children/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "structure_guard_children/plugin_importer.rs"]
mod plugin_importer;
#[path = "structure_guard_children/review_guard_groups.rs"]
mod review_guard_groups;
#[path = "structure_guard_children/status_docs.rs"]
mod status_docs;
#[path = "structure_guard_children/typed_error.rs"]
mod typed_error;

pub(super) const STRUCTURE_GUARD_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs";
pub(super) const STRUCTURE_GUARD_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs";
pub(super) const F8_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs";
pub(super) const F8_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/delegation.rs";
pub(super) const F8_ROUTE_OWNERSHIP_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership.rs";
pub(super) const F8_STATUS_MIRRORS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/status_mirrors.rs";
pub(super) const F8_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/budgets.rs";
pub(super) const LATE_API_CLEANUP_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners.rs";
pub(super) const LATE_API_CLEANUP_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/delegation.rs";
pub(super) const LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/route_ownership.rs";
pub(super) const LATE_API_CLEANUP_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/status_mirrors.rs";
pub(super) const LATE_API_CLEANUP_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/budgets.rs";
pub(super) const P0_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs";
pub(super) const P0_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/delegation.rs";
pub(super) const P0_ROUTE_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/route_ownership.rs";
pub(super) const P0_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/status_mirrors.rs";
pub(super) const P0_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/budgets.rs";
pub(super) const P0_NATIVE_FIXTURE_LEAF_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners.rs";
pub(super) const P0_NATIVE_FIXTURE_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/delegation.rs";
pub(super) const P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/leaf_ownership.rs";
pub(super) const P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/status_mirrors.rs";
pub(super) const P0_NATIVE_FIXTURE_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs";
pub(super) const PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs.rs";
pub(super) const TYPED_ERROR_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/delegation.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_mirrors.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/budgets.rs";
pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts.rs";
pub(super) const TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/delegation.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/child_ownership.rs";
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/status_mirrors.rs";
pub(super) const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs";
pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs";
pub(super) const STATUS_DOCS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs";
pub(super) const STATUS_DOCS_SOURCE_ANCHORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors.rs";
pub(super) const STATUS_DOCS_STATUS_ANCHORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings structure guard children folder-backed split";
pub(super) const STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_ID: &str =
    "runtime_15_code_review_findings_structure_guard_children_folder_backed_static_passed_cargo_deferred";

pub(super) const STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/delegation.rs",
        "runtime_15_code_review_findings_structure_guard_children_are_mounted",
    ),
    (
        "review_guard_groups",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/review_guard_groups.rs",
        "runtime_15_code_review_findings_structure_guard_review_groups_are_child_owned",
    ),
    (
        "plugin_importer",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
        "runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned",
    ),
    (
        "status_docs",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/status_docs.rs",
        "runtime_15_code_review_findings_structure_guard_status_docs_are_child_owned",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs",
        "runtime_15_code_review_findings_structure_guard_children_folder_backed_status_is_current",
    ),
];

pub(super) fn structure_guard_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn structure_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn review_guard_status_rows_source() -> String {
    super::status_docs::review_guard_status_rows_source()
}

pub(super) fn assert_nested_structure_children_are_mounted() {
    folder_backed_summary::assert_folder_backed_summary_structure_children_are_mounted();
    typed_error::assert_typed_error_structure_children_are_mounted();
}
