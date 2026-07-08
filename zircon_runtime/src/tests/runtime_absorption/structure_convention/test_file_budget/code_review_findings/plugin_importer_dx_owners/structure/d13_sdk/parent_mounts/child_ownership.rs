use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_d13_sdk_parent_mounts_guard_is_child_backed() {
    let sources = plugin_importer_d13_sdk_structure_sources();
    let child_blob = plugin_importer_d13_sdk_parent_mount_child_source_blob();

    assert_contains_all(
        "plugin-importer D13 parent-mount guard routes focused child owners",
        &sources.parent_mounts_child,
        &[
            "#[path = \"parent_mounts/delegation.rs\"]",
            "#[path = \"parent_mounts/review_mounts.rs\"]",
            "#[path = \"parent_mounts/folder_backed.rs\"]",
            "#[path = \"parent_mounts/child_ownership.rs\"]",
            "#[path = \"parent_mounts/status_mirrors.rs\"]",
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_DELEGATION_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_REVIEW_MOUNTS_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_FOLDER_BACKED_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_OWNERSHIP_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_MIRRORS_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_STATUS,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_GUARD,
        ],
    );
    for moved_body in [
        "plugin-importer DX structure assertions delegate D13 SDK structure checks to child owner",
        "plugin importer DX D13 parent mounts focused SDK review guard children",
        "plugin-importer D13 SDK structure assertions route owns focused child inventory",
        "plugin-importer D13 SDK structure assertions parent records folder-backed status",
    ] {
        assert!(
            !sources.parent_mounts_child.contains(moved_body),
            "parent_mounts.rs should delegate moved assertion body `{moved_body}` to focused children"
        );
    }
    for (_, child_path, child_guard) in PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILDREN {
        assert!(
            sources.parent_mounts_child.contains(child_path),
            "parent_mounts.rs should inventory child owner path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "parent_mounts child source blob should contain child guard {child_guard}"
        );
    }
    super::status_mirrors::assert_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current(
    );
}
