use super::super::super::*;
use super::*;

#[path = "plugin_importer/child_ownership.rs"]
mod child_ownership;
#[path = "plugin_importer/source_inventory.rs"]
mod source_inventory;
#[path = "plugin_importer/status_docs.rs"]
mod status_docs;
#[path = "plugin_importer/status_mirrors.rs"]
mod status_mirrors;
#[path = "plugin_importer/structure_assertions.rs"]
mod structure_assertions;
#[path = "plugin_importer/top_level_children.rs"]
mod top_level_children;

pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_TOP_LEVEL_CHILDREN_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/top_level_children.rs";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/structure_assertions.rs";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/source_inventory.rs";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_DOCS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_docs.rs";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/child_ownership.rs";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_mirrors.rs";

pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_SLICE: &str =
    "Runtime 15 M3 structure guard plugin-importer child split";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_STATUS: &str =
    "runtime_15_structure_guard_plugin_importer_child_split_static_passed_cargo_deferred";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_DATE: &str = "2026-07-05";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_GUARD: &str =
    "runtime_15_structure_guard_plugin_importer_is_child_backed";
pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRROR_GUARD: &str =
    "runtime_15_structure_guard_plugin_importer_status_mirrors_are_current";

pub(super) const STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "top_level_children",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_TOP_LEVEL_CHILDREN_CHILD,
        "assert_plugin_importer_dx_top_level_children_are_mounted",
    ),
    (
        "structure_assertions",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_CHILD,
        "assert_plugin_importer_dx_structure_assertions_are_mounted",
    ),
    (
        "source_inventory",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_SOURCE_INVENTORY_CHILD,
        "assert_plugin_importer_dx_source_inventory_is_mounted",
    ),
    (
        "status_docs",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_DOCS_CHILD,
        "assert_plugin_importer_dx_status_docs_are_mounted",
    ),
    (
        "child_ownership",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_OWNERSHIP_CHILD,
        STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_GUARD,
    ),
    (
        "status_mirrors",
        STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRRORS_CHILD,
        STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRROR_GUARD,
    ),
];

pub(super) fn plugin_importer_structure_guard_child_source_blob() -> String {
    let mut blob = String::new();
    blob.push_str(&read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
    ));
    blob.push('\n');
    for (_, child_path, _) in STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILDREN {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(super) fn assert_plugin_importer_dx_children_are_mounted() {
    top_level_children::assert_plugin_importer_dx_top_level_children_are_mounted();
    structure_assertions::assert_plugin_importer_dx_structure_assertions_are_mounted();
    source_inventory::assert_plugin_importer_dx_source_inventory_is_mounted();
    status_docs::assert_plugin_importer_dx_status_docs_are_mounted();
}

#[test]
fn runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned() {
    assert_plugin_importer_dx_children_are_mounted();
}

#[test]
fn runtime_15_structure_guard_plugin_importer_is_child_backed() {
    child_ownership::assert_structure_guard_plugin_importer_is_child_backed();
}

#[test]
fn runtime_15_structure_guard_plugin_importer_status_mirrors_are_current() {
    status_mirrors::assert_structure_guard_plugin_importer_status_mirrors_are_current();
}
