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

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/paths.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/sources.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/parent_mounts.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/review_children.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/review_mounts/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_STATUS: &str =
    "runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_dx_review_mounts_guard_is_folder_backed";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGET_GUARD: &str =
    "runtime_15_plugin_importer_dx_review_mounts_children_line_budgets_are_current";
pub(super) const PLUGIN_IMPORTER_DX_REVIEW_CHILDREN_GUARD: &str =
    "assert_plugin_importer_dx_review_children_are_mounted";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

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
];

pub(super) struct PluginImporterDxReviewMountSources {
    pub(super) structure_assertions_child: String,
    pub(super) review_mounts_child: String,
    pub(super) paths_child: String,
    pub(super) sources_child: String,
    pub(super) parent_mounts_child: String,
    pub(super) review_children_child: String,
    pub(super) budgets_child: String,
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
    PluginImporterDxReviewMountSources {
        structure_assertions_child: read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD),
        review_mounts_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD),
        paths_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD),
        sources_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD),
        parent_mounts_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD),
        review_children_child: read_runtime_src(
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD,
        ),
        budgets_child: read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD),
        plugin_importer_dx: read_runtime_src(paths::PLUGIN_IMPORTER_DX_REVIEW_SOURCE_PATH),
        plugin_importer_dx_d10: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D10_SOURCE_PATH),
        plugin_importer_dx_d1: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D1_SOURCE_PATH),
        plugin_importer_dx_d1_children: paths::PLUGIN_IMPORTER_DX_D1_CHILD_SOURCE_PATHS
            .iter()
            .map(|path| read_runtime_src(path))
            .collect::<Vec<_>>()
            .join("\n"),
        plugin_importer_dx_d11: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D11_SOURCE_PATH),
        plugin_importer_dx_d12: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D12_SOURCE_PATH),
        plugin_importer_dx_d5: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D5_SOURCE_PATH),
        plugin_importer_dx_d6: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D6_SOURCE_PATH),
        plugin_importer_dx_d8: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D8_SOURCE_PATH),
        plugin_importer_dx_d9: read_runtime_src(paths::PLUGIN_IMPORTER_DX_D9_SOURCE_PATH),
    }
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
