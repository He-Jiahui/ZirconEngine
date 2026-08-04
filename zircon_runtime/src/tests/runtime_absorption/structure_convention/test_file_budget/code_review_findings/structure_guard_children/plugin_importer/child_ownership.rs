use super::super::*;
use super::*;

pub(super) fn assert_structure_guard_plugin_importer_is_child_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
    );
    let child_blob = plugin_importer_structure_guard_child_source_blob();

    assert_contains_all(
        "structure guard plugin-importer route mounts focused child owners",
        &parent,
        &[
            "#[path = \"plugin_importer/top_level_children.rs\"]",
            "#[path = \"plugin_importer/structure_assertions.rs\"]",
            "#[path = \"plugin_importer/source_inventory.rs\"]",
            "#[path = \"plugin_importer/status_docs.rs\"]",
            "#[path = \"plugin_importer/child_ownership.rs\"]",
            "#[path = \"plugin_importer/status_mirrors.rs\"]",
            STRUCTURE_GUARD_PLUGIN_IMPORTER_TOP_LEVEL_CHILDREN_CHILD,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_CHILD,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_SOURCE_INVENTORY_CHILD,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_OWNERSHIP_CHILD,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_STATUS,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_GUARD,
        ],
    );
    for moved_body in [
        "plugin-importer DX structure child owner keeps plugin DX review guard ownership checks",
        "plugin-importer DX structure assertions parent keeps focused guard mounts",
        "plugin-importer DX source inventory child keeps DX source-path checks",
        "plugin-importer DX status-doc child keeps docs/status checks",
    ] {
        assert!(
            !parent.contains(moved_body),
            "plugin_importer.rs should delegate moved assertion body `{moved_body}` to focused children"
        );
    }
    for (_, child_path, child_guard) in STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILDREN {
        assert!(
            parent.contains(child_path),
            "plugin_importer.rs should inventory child owner path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "structure guard plugin-importer child source blob should contain child guard {child_guard}"
        );
    }
}
