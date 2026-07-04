use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_plugin_importer_d13_sdk_structure_parent_delegates(
    sources: &PluginImporterD13SdkStructureSources,
) {
    assert_contains_all(
        "plugin-importer DX structure assertions delegate D13 SDK structure checks to child owner",
        &sources.structure_assertions_child,
        &[
            "#[path = \"structure_assertions/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );
}
