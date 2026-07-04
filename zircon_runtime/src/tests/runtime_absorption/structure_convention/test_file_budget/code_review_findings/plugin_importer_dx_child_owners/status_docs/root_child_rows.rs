use super::*;

pub(super) const PLUGIN_IMPORTER_DX_STATUS_DOC_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        PLUGIN_IMPORTER_DX_STATUS_DOC_DELEGATION_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_GUARD,
    ),
    (
        "doc_mirrors",
        PLUGIN_IMPORTER_DX_STATUS_DOC_DOC_MIRRORS_OWNER,
        "assert_plugin_importer_dx_status_doc_mirrors_are_synced",
    ),
    (
        "status_maps",
        PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS_MAPS_OWNER,
        "assert_plugin_importer_dx_status_maps_are_synced",
    ),
    (
        "status_mirrors",
        PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS_MIRRORS_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_STATUS_GUARD,
    ),
];

pub(super) const PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_PATHS_OWNER,
        "PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_PATHS_OWNER",
    ),
    (
        "root_statuses",
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_STATUSES_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILD_ROWS_OWNER,
        "PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_SOURCES_OWNER,
        "plugin_importer_dx_status_doc_sources",
    ),
    (
        "root_inventory",
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_OWNER,
        PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_INVENTORY_GUARD,
    ),
];
