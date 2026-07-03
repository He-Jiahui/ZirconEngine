use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_assertions_are_child_owner() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let child = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD);
    let child_tree = plugin_importer_dx_structure_assertion_child_source_blob();

    assert_contains_all(
        "plugin-importer DX structure guard delegates structure assertions to child owner",
        &parent,
        &[
            "#[path = \"plugin_importer_dx_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed",
        ],
    );
    for moved_anchor in [
        "let plugin_importer_dx = read_runtime_src(",
        "let plugin_importer_dx_d10 = read_runtime_src(",
        "let plugin_importer_dx_d13 = read_runtime_src(",
        "fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        concat!(
            "fn review_d13_importer_manifest_parity_guard_",
            "lives_in_sdk_builder"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "plugin_importer_dx_child_owners.rs should delegate structure assertion anchor `{moved_anchor}` to structure_assertions.rs"
        );
    }
    assert_contains_all(
        "plugin-importer DX structure assertions parent delegates focused guard children",
        &child,
        &[
            "#[path = \"structure_assertions/review_mounts.rs\"]",
            "mod review_mounts;",
            "#[path = \"structure_assertions/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure_assertions/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure_assertions/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"structure_assertions/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed",
            "review_mounts::assert_plugin_importer_dx_review_mounts_are_folder_backed",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertion children own review guard structure checks",
        &child_tree,
        &[
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "review_d10_animation_physics_tests_use_sdk_bridge_call",
            "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "review_d5_editor_authoring_plugins_use_sdk_macro",
            "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "runtime_15_plugin_importer_dx_structure_assertions_are_child_owner",
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
        ],
    );
    for (_, child_path, anchor) in PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTION_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "plugin-importer DX structure assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "plugin-importer DX structure assertion child {child_path} should own anchor {anchor}"
        );
    }

    assert_plugin_importer_dx_child_owners_are_folder_backed();

    for (path, source) in [
        (PLUGIN_IMPORTER_DX_STRUCTURE_CHILD, parent),
        (PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD, child),
    ]
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
