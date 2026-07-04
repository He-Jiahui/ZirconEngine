use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    (
        "child_ownership",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "source_inventory",
        PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
        "runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
    ),
    (
        "structure_assertions",
        PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed",
    ),
    (
        "status_docs",
        PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD,
        "runtime_15_plugin_importer_dx_status_docs_are_child_owner",
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD,
        BUDGET_GUARD,
    ),
];

pub(super) const PLUGIN_IMPORTER_DX_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        PLUGIN_IMPORTER_DX_ROOT_PATHS_CHILD,
        "PLUGIN_IMPORTER_DX_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        PLUGIN_IMPORTER_DX_ROOT_STATUSES_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        PLUGIN_IMPORTER_DX_ROOT_CHILD_ROWS_CHILD,
        "PLUGIN_IMPORTER_DX_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        PLUGIN_IMPORTER_DX_ROOT_SOURCES_CHILD,
        "plugin_importer_dx_structure_status_row_source",
    ),
    (
        "root_inventory",
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_CHILD,
        PLUGIN_IMPORTER_DX_ROOT_INVENTORY_GUARD,
    ),
];
