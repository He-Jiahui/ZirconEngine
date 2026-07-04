use super::super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_status_maps_are_synced(
    sources: &PluginImporterDxStatusDocSources,
) {
    assert_contains_all(
        "status/date expected-slice maps",
        &sources.status_maps,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_SLICE,
            PLUGIN_IMPORTER_DX_STATUS_DOC_STATUS,
            "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split",
            "runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_STATUS,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_SLICE,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_FOLDER_STATUS,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_SLICE,
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_STATUS,
            "2026-07-02",
            "2026-06-30",
        ],
    );
}
