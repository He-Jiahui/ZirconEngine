use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_budgets_are_focused() {
    let sources = [
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD),
        ),
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD),
        ),
        (
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
            read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD),
        ),
    ];

    for (path, source) in sources {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in folder_backed_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused plugin-importer DX structure child budget; got {line_count} lines"
        );
    }
}
