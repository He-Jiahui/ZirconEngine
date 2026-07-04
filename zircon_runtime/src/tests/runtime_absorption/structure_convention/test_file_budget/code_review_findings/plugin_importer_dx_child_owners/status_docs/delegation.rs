use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_status_docs_are_folder_backed() {
    let plugin_importer_parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNER);
    let status_docs_parent = read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER);
    let child_inventory = read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOC_ROOT_CHILD_ROWS_OWNER);
    let status_docs_child_tree = plugin_importer_dx_status_docs_child_source_blob();

    assert_contains_all(
        "plugin-importer DX structure owner delegates status-doc sync to child owner",
        &plugin_importer_parent,
        &[
            "#[path = \"plugin_importer_dx_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "status_docs::assert_plugin_importer_dx_status_docs_are_synced",
        ],
    );
    for moved_anchor in [
        "let runtime_15_plan =",
        "let status_rows = format!(",
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split",
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
    ] {
        assert!(
            !plugin_importer_parent.contains(moved_anchor),
            "plugin-importer DX status-doc anchor `{moved_anchor}` should stay in {PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER}"
        );
    }
    assert_contains_all(
        "plugin-importer DX status-doc parent delegates focused guard children",
        &status_docs_parent,
        &[
            "#[path = \"status_docs/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"status_docs/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"status_docs/status_maps.rs\"]",
            "mod status_maps;",
            "#[path = \"status_docs/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"status_docs/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"status_docs/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"status_docs/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"status_docs/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"status_docs/root_inventory.rs\"]",
            "mod root_inventory;",
            "pub(super) fn assert_plugin_importer_dx_status_docs_are_synced",
            "doc_mirrors::assert_plugin_importer_dx_status_doc_mirrors_are_synced",
            "status_maps::assert_plugin_importer_dx_status_maps_are_synced",
            "plugin_importer_dx_status_doc_sources",
        ],
    );
    assert_contains_all(
        "plugin-importer DX status-doc children own delegated assertions",
        &status_docs_child_tree,
        &[
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_GUARD,
            "assert_plugin_importer_dx_status_doc_mirrors_are_synced",
            "assert_plugin_importer_dx_status_maps_are_synced",
            PLUGIN_IMPORTER_DX_STATUS_DOC_FOLDER_BACKED_STATUS_GUARD,
        ],
    );
    for (module_name, child_path, anchor) in PLUGIN_IMPORTER_DX_STATUS_DOC_CHILDREN {
        let path_attr = format!("#[path = \"status_docs/{module_name}.rs\"]");
        assert!(
            status_docs_parent.contains(&path_attr),
            "plugin-importer DX status-doc parent should mount {module_name}"
        );
        assert!(
            child_inventory.contains(child_path),
            "plugin-importer DX status-doc child inventory should list {child_path}"
        );
        assert!(
            status_docs_child_tree.contains(anchor),
            "plugin-importer DX status-doc child {child_path} should own anchor {anchor}"
        );
    }

    assert_plugin_importer_dx_status_docs_are_synced();

    for (path, source) in [(PLUGIN_IMPORTER_DX_STATUS_DOC_OWNER, status_docs_parent)]
        .into_iter()
        .chain(plugin_importer_dx_status_docs_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_STATUS_DOC_CHILD_LINE_BUDGET,
            "{path} should stay below the focused plugin-importer DX status-doc budget; got {line_count} lines"
        );
    }
}
