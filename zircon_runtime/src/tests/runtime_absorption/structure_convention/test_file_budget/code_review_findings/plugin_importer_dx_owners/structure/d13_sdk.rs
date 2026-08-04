use super::super::super::super::*;

#[path = "d13_sdk/budgets.rs"]
mod budgets;
#[path = "d13_sdk/parent_mounts.rs"]
mod parent_mounts;
#[path = "d13_sdk/paths.rs"]
mod paths;
#[path = "d13_sdk/review_children.rs"]
mod review_children;
#[path = "d13_sdk/sources.rs"]
mod sources;

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure_assertions.rs";
pub(super) const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk.rs";
pub(super) const PLUGIN_IMPORTER_D13_PATHS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/paths.rs";
pub(super) const PLUGIN_IMPORTER_D13_SOURCES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/sources.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/parent_mounts.rs";
pub(super) const PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/review_children.rs";
pub(super) const PLUGIN_IMPORTER_D13_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure/d13_sdk/budgets.rs";
pub(super) const PLUGIN_IMPORTER_D13_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split";
pub(super) const PLUGIN_IMPORTER_D13_FOLDER_BACKED_STATUS: &str = "runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const PLUGIN_IMPORTER_D13_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const PLUGIN_IMPORTER_D13_FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed";
pub(super) const PLUGIN_IMPORTER_D13_BUDGET_GUARD: &str =
    "runtime_15_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current";
pub(super) const PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_GUARD: &str =
    "assert_plugin_importer_d13_sdk_review_children_are_child_owned";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTION_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "paths",
        PLUGIN_IMPORTER_D13_PATHS_CHILD,
        "pub(super) const PLUGIN_IMPORTER_D13_SOURCE_PATH",
    ),
    (
        "sources",
        PLUGIN_IMPORTER_D13_SOURCES_CHILD,
        "pub(super) fn plugin_importer_d13_sdk_structure_sources",
    ),
    (
        "parent_mounts",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD,
        PLUGIN_IMPORTER_D13_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_CHILD,
        PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_GUARD,
    ),
    (
        "budgets",
        PLUGIN_IMPORTER_D13_BUDGETS_CHILD,
        PLUGIN_IMPORTER_D13_BUDGET_GUARD,
    ),
];

pub(super) struct PluginImporterD13SdkStructureSources {
    pub(super) structure_assertions_child: String,
    pub(super) d13_sdk_child: String,
    pub(super) paths_child: String,
    pub(super) sources_child: String,
    pub(super) parent_mounts_child: String,
    pub(super) review_children_child: String,
    pub(super) budgets_child: String,
    pub(super) status_mirrors_child: String,
    pub(super) plugin_importer_dx_d13: String,
    pub(super) plugin_importer_dx_d13_manifest_parity: String,
    pub(super) plugin_importer_dx_d13_runtime_crates: String,
    pub(super) plugin_importer_dx_d13_runtime_exports: String,
    pub(super) plugin_importer_dx_d13_runtime_manifests: String,
}

pub(super) fn assert_plugin_importer_d13_sdk_child_owners_are_folder_backed() {
    let sources = plugin_importer_d13_sdk_structure_sources();

    parent_mounts::assert_plugin_importer_d13_sdk_parent_mounts_review_children(&sources);
    review_children::assert_plugin_importer_d13_sdk_review_children_are_child_owned(&sources);
}

pub(super) fn plugin_importer_d13_sdk_structure_sources() -> PluginImporterD13SdkStructureSources {
    sources::plugin_importer_d13_sdk_structure_sources()
}

pub(super) fn plugin_importer_d13_sdk_structure_child_sources() -> Vec<(&'static str, String)> {
    PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTION_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_d13_sdk_structure_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in plugin_importer_d13_sdk_structure_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
