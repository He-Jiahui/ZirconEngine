use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD);
    let review_mounts_child = read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD);
    let review_mounts_child_tree = format!(
        "{}\n{}",
        review_mounts_child,
        review_mounts::plugin_importer_dx_review_mount_child_source_blob()
    );
    let child_tree = plugin_importer_dx_structure_assertion_child_source_blob();

    for child_owned_guard in [
        "let plugin_importer_dx = read_runtime_src(",
        "let plugin_importer_dx_d10 = read_runtime_src(",
        "let plugin_importer_dx_d13 = read_runtime_src(",
        "fn review_d10_animation_physics_tests_use_sdk_bridge_call",
        "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
    ] {
        assert!(
            !parent.contains(child_owned_guard),
            "plugin-importer DX structure assertion guard `{child_owned_guard}` should stay in a focused child"
        );
    }
    assert_contains_all(
        "plugin-importer DX structure assertions parent mounts focused guard children",
        &parent,
        &[
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/review_mounts.rs\"]",
            "mod review_mounts;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "plugin_importer_dx_structure_assertion_child_sources",
            "plugin_importer_dx_structure_assertion_child_source_blob",
        ],
    );
    assert_contains_all(
        "plugin-importer DX review mounts child owns non-D13 structure checks",
        &review_mounts_child_tree,
        &[
            "pub(super) fn assert_plugin_importer_dx_review_mounts_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d5_editor_authoring.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d8_registration_builder.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
            "review_d10_animation_physics_tests_use_sdk_bridge_call",
            "review_d5_editor_authoring_plugins_use_sdk_macro",
            "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertion guard children own delegated assertions",
        &child_tree,
        &[
            "runtime_15_plugin_importer_dx_structure_assertions_are_child_owner",
            "runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned",
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
            "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current",
        ],
    );

    assert_plugin_importer_dx_child_owners_are_folder_backed();

    for (path, source) in [(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD, parent)]
        .into_iter()
        .chain(plugin_importer_dx_structure_assertion_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
