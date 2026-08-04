use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_parent_mounts_review_children(
    sources: &PluginImporterDxReviewMountSources,
) {
    assert_contains_all(
        "plugin-importer DX structure assertions delegate review mounts to child owner",
        &sources.structure_assertions_child,
        &[
            "#[path = \"structure/review_mounts.rs\"]",
            "mod review_mounts;",
            "review_mounts::assert_plugin_importer_dx_review_mounts_are_folder_backed",
        ],
    );
    assert_contains_all(
        "plugin importer DX parent mounts importer review guard children",
        &sources.plugin_importer_dx,
        &[
            "#[path = \"plugin_importer_dx/d10_bridge_call.rs\"]",
            "mod d10_bridge_call;",
            "#[path = \"plugin_importer_dx/d1_capability_single_source.rs\"]",
            "mod d1_capability_single_source;",
            "#[path = \"plugin_importer_dx/d11_test_runtime_fixture.rs\"]",
            "mod d11_test_runtime_fixture;",
            "#[path = \"plugin_importer_dx/d12_runtime_exports.rs\"]",
            "mod d12_runtime_exports;",
            "#[path = \"plugin_importer_dx/d13_importer_sdk.rs\"]",
            "mod d13_importer_sdk;",
            "#[path = \"plugin_importer_dx/d6_runtime_plugin_id.rs\"]",
            "mod d6_runtime_plugin_id;",
            "#[path = \"plugin_importer_dx/d5_editor_authoring.rs\"]",
            "mod d5_editor_authoring;",
            "#[path = \"plugin_importer_dx/d8_registration_builder.rs\"]",
            "mod d8_registration_builder;",
            "#[path = \"plugin_importer_dx/d9_editor_runtime_mirror.rs\"]",
            "mod d9_editor_runtime_mirror;",
        ],
    );
    assert_eq!(
        sources.plugin_importer_dx.matches("#[test]").count(),
        0,
        "plugin_importer_dx.rs should only mount child review guard owners"
    );
    for child_owned_test in [
        "fn review_d10_animation_physics_tests_use_sdk_bridge_call",
        "fn review_d6_runtime_plugin_id_accepts_external_string_keys",
        "fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        "fn review_d5_editor_authoring_plugins_use_sdk_macro",
        "fn review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
    ] {
        assert!(
            !sources.plugin_importer_dx.contains(child_owned_test),
            "child-owned plugin-importer DX review guard `{child_owned_test}` should not return to plugin_importer_dx.rs"
        );
    }
}

#[test]
fn runtime_15_plugin_importer_dx_review_mounts_guard_is_folder_backed() {
    let sources = plugin_importer_dx_review_mount_sources();
    let child_blob = plugin_importer_dx_review_mount_child_source_blob();

    assert_plugin_importer_dx_parent_mounts_review_children(&sources);
    review_children::assert_plugin_importer_dx_review_children_are_mounted(&sources);
    budgets::assert_plugin_importer_dx_review_mounts_children_line_budgets_are_current(&sources);
    for (_, child_path, child_guard) in PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILDREN {
        assert!(
            sources.review_mounts_child.contains(child_path),
            "plugin-importer DX review mounts parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "plugin-importer DX review mounts child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !sources
            .review_mounts_child
            .contains("let plugin_importer_dx_d10 = read_runtime_src"),
        "review_mounts.rs should delegate review child source reads to sources.rs"
    );
    assert_contains_all(
        "plugin-importer DX review mounts parent records folder-backed status",
        &sources.review_mounts_child,
        &[
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_STATUS,
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_FOLDER_BACKED_GUARD,
        ],
    );
}
