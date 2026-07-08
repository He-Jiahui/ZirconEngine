use super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_top_level_children_are_mounted() {
    let plugin_importer_dx_child = read_runtime_src(PLUGIN_IMPORTER_DX_CHILD_OWNER);
    let plugin_importer_dx_top_level_delegation_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER);
    let plugin_importer_dx_top_level_child_ownership_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER);
    let plugin_importer_dx_top_level_status_mirrors_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER);
    let plugin_importer_dx_top_level_budgets_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER);
    let plugin_importer_dx_source_inventory_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER);
    let plugin_importer_dx_child_tree = format!(
        "{}\n{}\n{}\n{}\n{}",
        plugin_importer_dx_child,
        read_runtime_src(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/root_statuses.rs",
        ),
        super::super::super::plugin_importer_dx_child_owners::folder_backed_child_source_blob(),
        [
            PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER,
            PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
            PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER,
            PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER,
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER,
        ]
        .join("\n"),
        [
            plugin_importer_dx_top_level_delegation_child.as_str(),
            plugin_importer_dx_top_level_child_ownership_child.as_str(),
            plugin_importer_dx_top_level_status_mirrors_child.as_str(),
            plugin_importer_dx_top_level_budgets_child.as_str(),
            plugin_importer_dx_source_inventory_child.as_str(),
        ]
        .join("\n")
    );

    assert_contains_all(
        "plugin-importer DX structure child owner keeps plugin DX review guard ownership checks",
        &plugin_importer_dx_child_tree,
        &[
            "Runtime 15 M3 plugin-importer DX structure guard folder-backed split",
            "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_structure_guard_is_folder_backed",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/delegation.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/child_ownership.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/status_mirrors.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/budgets.rs",
            "#[path = \"plugin_importer_dx_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"plugin_importer_dx_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"plugin_importer_dx_owners/status_docs.rs\"]",
            "mod status_docs;",
            "fn runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner",
            "structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "source_inventory::plugin_importer_dx_review_guard_count",
            "status_docs::assert_plugin_importer_dx_status_docs_are_synced",
        ],
    );
}
