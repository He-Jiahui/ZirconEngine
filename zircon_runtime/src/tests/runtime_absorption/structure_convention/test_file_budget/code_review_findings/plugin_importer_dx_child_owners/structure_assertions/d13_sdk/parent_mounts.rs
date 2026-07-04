use super::super::super::super::super::*;
use super::*;

#[path = "parent_mounts/child_ownership.rs"]
mod child_ownership;
#[path = "parent_mounts/delegation.rs"]
mod delegation;
#[path = "parent_mounts/folder_backed.rs"]
mod folder_backed;
#[path = "parent_mounts/review_mounts.rs"]
mod review_mounts;
#[path = "parent_mounts/status_mirrors.rs"]
mod status_mirrors;

pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/delegation.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_REVIEW_MOUNTS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/review_mounts.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_FOLDER_BACKED_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/folder_backed.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/child_ownership.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/status_mirrors.rs";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_SLICE: &str =
    "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_STATUS: &str =
    "runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_child_split_static_passed_cargo_deferred";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_DATE: &str = "2026-07-05";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_GUARD: &str =
    "runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_is_child_backed";
pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_GUARD: &str =
    "runtime_15_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current";

pub(super) const PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_DELEGATION_CHILD,
        "assert_plugin_importer_d13_sdk_structure_parent_delegates",
    ),
    (
        "review_mounts",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_REVIEW_MOUNTS_CHILD,
        "assert_plugin_importer_d13_sdk_parent_mounts_review_children",
    ),
    (
        "folder_backed",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_FOLDER_BACKED_CHILD,
        "assert_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed",
    ),
    (
        "child_ownership",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_OWNERSHIP_CHILD,
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_GUARD,
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_MIRRORS_CHILD,
        PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_GUARD,
    ),
];

pub(super) fn assert_plugin_importer_d13_sdk_structure_parent_delegates(
    sources: &PluginImporterD13SdkStructureSources,
) {
    delegation::assert_plugin_importer_d13_sdk_structure_parent_delegates(sources);
}

pub(super) fn assert_plugin_importer_d13_sdk_parent_mounts_review_children(
    sources: &PluginImporterD13SdkStructureSources,
) {
    review_mounts::assert_plugin_importer_d13_sdk_parent_mounts_review_children(sources);
}

pub(super) fn plugin_importer_d13_sdk_parent_mount_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_path, _) in PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILDREN {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

#[test]
fn runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner() {
    folder_backed::assert_plugin_importer_d13_sdk_structure_assertions_are_child_owner();
}

#[test]
fn runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed() {
    folder_backed::assert_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed();
}

#[test]
fn runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_is_child_backed() {
    child_ownership::assert_plugin_importer_d13_sdk_parent_mounts_guard_is_child_backed();
}
