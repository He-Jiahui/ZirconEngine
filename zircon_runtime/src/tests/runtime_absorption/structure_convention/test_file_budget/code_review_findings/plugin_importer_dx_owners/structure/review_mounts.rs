use super::super::super::super::*;

#[path = "review_mounts/budgets.rs"]
mod budgets;
#[path = "review_mounts/parent_mounts.rs"]
mod parent_mounts;
#[path = "review_mounts/paths.rs"]
mod paths;
#[path = "review_mounts/review_children.rs"]
mod review_children;
#[path = "review_mounts/sources.rs"]
mod sources;
#[path = "review_mounts/status_mirrors.rs"]
mod status_mirrors;

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/paths.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/sources.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/parent_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/review_children.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_dx_review_mounts_guard_is_folder_backed";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_GUARD: &str =
    "runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_status_is_current";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGET_GUARD: &str =
    "runtime_15_plugin_importer_dx_review_mounts_children_line_budgets_are_current";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_CHILDREN_GUARD: &str =
    "assert_plugin_importer_dx_review_children_are_mounted";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs";
const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/plugin_importer_maps.rs";
const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/plugin_importer_maps.rs";

pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "paths",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD,
        "pub(super) const PLUGIN_IMPORTER_DX_REVIEW_SOURCE_PATH",
    ),
    (
        "sources",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD,
        "pub(super) fn plugin_importer_dx_review_mount_sources",
    ),
    (
        "parent_mounts",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD,
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD,
        PLUGIN_IMPORTER_DX_REVIEW_CHILDREN_GUARD,
    ),
    (
        "budgets",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD,
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGET_GUARD,
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_MIRRORS_CHILD,
        PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_GUARD,
    ),
];

pub(super) struct PluginImporterDxReviewMountSources {
    pub(super) structure_assertions_child: String,
    pub(super) review_mounts_child: String,
    pub(super) paths_child: String,
    pub(super) sources_child: String,
    pub(super) parent_mounts_child: String,
    pub(super) review_children_child: String,
    pub(super) budgets_child: String,
    pub(super) status_mirrors_child: String,
    pub(super) plugin_importer_dx: String,
    pub(super) plugin_importer_dx_d10: String,
    pub(super) plugin_importer_dx_d1: String,
    pub(super) plugin_importer_dx_d1_children: String,
    pub(super) plugin_importer_dx_d11: String,
    pub(super) plugin_importer_dx_d12: String,
    pub(super) plugin_importer_dx_d5: String,
    pub(super) plugin_importer_dx_d6: String,
    pub(super) plugin_importer_dx_d8: String,
    pub(super) plugin_importer_dx_d9: String,
}

pub(super) fn assert_plugin_importer_dx_review_mounts_are_folder_backed() {
    let sources = plugin_importer_dx_review_mount_sources();

    parent_mounts::assert_plugin_importer_dx_parent_mounts_review_children(&sources);
    review_children::assert_plugin_importer_dx_review_children_are_mounted(&sources);
}

pub(super) fn plugin_importer_dx_review_mount_sources() -> PluginImporterDxReviewMountSources {
    sources::plugin_importer_dx_review_mount_sources()
}

pub(super) fn plugin_importer_dx_review_mount_child_sources() -> Vec<(&'static str, String)> {
    PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_review_mount_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in plugin_importer_dx_review_mount_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
