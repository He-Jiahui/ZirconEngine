use super::super::super::*;

#[path = "sources/budgets.rs"]
mod budgets;
#[path = "sources/delegation.rs"]
mod delegation;
#[path = "sources/paths.rs"]
mod paths;
#[path = "sources/reads.rs"]
mod reads;

pub(super) const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/source_inventory.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_PATHS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/paths.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_READS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/reads.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/budgets.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/sources/delegation.rs";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_STATUS: &str = "runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_GUARD: &str =
    "runtime_15_plugin_importer_dx_source_inventory_guard_is_folder_backed";
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGET_GUARD: &str =
    "runtime_15_plugin_importer_dx_source_inventory_children_line_budgets_are_current";
pub(super) const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "paths",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_PATHS_CHILD,
        "const PLUGIN_IMPORTER_DX_SOURCE_PATHS",
    ),
    (
        "reads",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_READS_CHILD,
        "pub(super) fn plugin_importer_dx_sources",
    ),
    (
        "budgets",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGETS_CHILD,
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGET_GUARD,
    ),
    (
        "delegation",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_DELEGATION_CHILD,
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_GUARD,
    ),
];

pub(super) struct PluginImporterDxSourceInventorySources {
    pub(super) structure_child: String,
    pub(super) source_inventory_child: String,
    pub(super) paths_child: String,
    pub(super) reads_child: String,
    pub(super) budgets_child: String,
    pub(super) delegation_child: String,
    pub(super) status_mirrors_child: String,
}

pub(super) fn assert_plugin_importer_dx_line_budgets() {
    budgets::assert_plugin_importer_dx_line_budgets();
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    reads::plugin_importer_dx_review_guard_count()
}

pub(super) fn plugin_importer_dx_source_inventory_child_sources() -> Vec<(&'static str, String)> {
    PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn plugin_importer_dx_source_inventory_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in plugin_importer_dx_source_inventory_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
