use super::*;

pub(super) fn assert_plugin_importer_dx_line_budgets() {
    for (path, source) in super::reads::plugin_importer_dx_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the plugin-importer DX child-owner budget; got {line_count} lines"
        );
    }
}

pub(super) fn assert_plugin_importer_dx_source_inventory_children_line_budgets_are_current(
    sources: &PluginImporterDxSourceInventorySources,
) {
    for (path, source) in [
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_CHILD,
            sources.structure_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
            sources.source_inventory_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_PATHS_CHILD,
            sources.paths_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_READS_CHILD,
            sources.reads_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGETS_CHILD,
            sources.budgets_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_DELEGATION_CHILD,
            sources.delegation_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_plugin_importer_dx_source_inventory_children_line_budgets_are_current() {
    assert_plugin_importer_dx_line_budgets();
}
