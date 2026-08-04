use super::super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_source_inventory_is_child_owner(
    sources: &PluginImporterDxSourceInventorySources,
) {
    assert_contains_all(
        "plugin-importer DX structure guard delegates source inventory to child owner",
        &sources.structure_child,
        &[
            "#[path = \"plugin_importer_dx_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "source_inventory::assert_plugin_importer_dx_line_budgets",
            "source_inventory::plugin_importer_dx_review_guard_count",
        ],
    );
    assert!(
        !sources
            .structure_child
            .contains("const PLUGIN_IMPORTER_DX_SOURCE_PATHS"),
        "plugin_importer_dx_child_owners.rs should not retain the plugin-importer DX source inventory"
    );
    assert!(
        !sources
            .structure_child
            .contains("fn plugin_importer_dx_sources()"),
        "plugin_importer_dx_child_owners.rs should delegate plugin-importer DX source reads to source_inventory.rs"
    );
    assert_contains_all(
        "plugin-importer DX source inventory child mounts focused owners",
        &sources.source_inventory_child,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"sources/paths.rs\"]",
            "mod paths;",
            "#[path = \"sources/reads.rs\"]",
            "mod reads;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "budgets::assert_plugin_importer_dx_line_budgets",
            "reads::plugin_importer_dx_review_guard_count",
        ],
    );
    assert_contains_all(
        "plugin-importer DX source inventory paths child owns source paths",
        &sources.paths_child,
        &[
            "const PLUGIN_IMPORTER_DX_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
            "pub(super) fn plugin_importer_dx_source_paths",
        ],
    );
    assert_contains_all(
        "plugin-importer DX source inventory reads child owns source aggregation helpers",
        &sources.reads_child,
        &[
            "pub(super) fn plugin_importer_dx_sources",
            "pub(super) fn plugin_importer_dx_review_guard_count",
        ],
    );
    assert_eq!(
        plugin_importer_dx_review_guard_count(),
        11,
        "plugin-importer DX source inventory should preserve all current D1/D5/D6/D8/D9/D10/D11/D12/D13 review guards"
    );
}

#[test]
fn runtime_15_plugin_importer_dx_source_inventory_guard_is_folder_backed() {
    let child_blob = plugin_importer_dx_source_inventory_child_source_blob();
}
