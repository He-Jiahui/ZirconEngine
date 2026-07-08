use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let parent_inventory = status_doc_route_inventory_source();
    let child = read_runtime_src(STATUS_DOC_SOURCE_ANCHORS_OWNER);
    let child_blob = source_anchors::status_doc_source_anchor_child_source_blob();

    assert_contains_all(
        "status-doc guard mounts source-anchor child owner",
        &parent_inventory,
        &[
            "#[path = \"status/source_anchors.rs\"]",
            "mod source_anchors;",
            "#[path = \"status/source_anchor_guard.rs\"]",
            "mod source_anchor_guard;",
            STATUS_DOC_SOURCE_ANCHORS_SLICE,
            STATUS_DOC_SOURCE_ANCHORS_STATUS,
            STATUS_DOC_SOURCE_ANCHORS_OWNER,
            STATUS_DOC_SOURCE_ANCHORS_GUARD,
            source_anchors::SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
            source_anchors::SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    for moved_anchor in [
        concat!(
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/",
            "asset_loaders/animation_binary.rs"
        ),
        concat!(
            "review_f5_native_live_host_registration_replay_",
            "uses_typed_error"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/runtime_manifests.rs"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status_docs.rs should delegate source anchor `{moved_anchor}` to status/source_anchors.rs"
        );
    }
    assert_contains_all(
        "source-anchor parent owns folder-backed child delegation",
        &child,
        &[
            "pub(super) fn assert_code_review_findings_status_doc_source_anchors",
            "mod review_sources;",
            "mod native_typed_error;",
            "mod runtime_surface;",
            "mod structure_owners;",
            "mod status_mirrors;",
            "assert_status_doc_source_anchor_children_are_mounted",
            "status_doc_source_anchor_child_source_blob",
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
        ],
    );
    assert_contains_all(
        "source-anchor children own status-doc long source anchors",
        &child_blob,
        &[
            concat!(
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/",
                "asset_loaders/animation_binary.rs"
            ),
            concat!(
                "review_f5_native_live_host_registration_replay_",
                "uses_typed_error"
            ),
            concat!(
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
                "d13_importer_sdk/runtime_manifests.rs"
            ),
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
        ],
    );
    source_anchors::assert_status_doc_source_anchor_children_are_mounted();
    assert_code_review_findings_status_docs_are_synced();

    for (path, source) in [
        (STATUS_DOC_PARENT_PATH, parent.as_str()),
        (STATUS_DOC_SOURCE_ANCHORS_OWNER, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in source_anchors::status_doc_source_anchor_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
