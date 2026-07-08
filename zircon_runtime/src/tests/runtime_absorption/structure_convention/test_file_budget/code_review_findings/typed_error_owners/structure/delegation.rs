use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_structure_assertions_are_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let child_tree = structure_assertion_guard_child_source_blob();

    assert_contains_all(
        "typed-error structure guard delegates structure assertions to child owner",
        &parent,
        &[
            "#[path = \"typed_error_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "structure_assertions::assert_typed_error_child_owners_are_folder_backed",
        ],
    );
    assert!(
        !parent.contains("let asset_loaders_parent = read_runtime_src"),
        "typed_error_child_owners.rs should not retain asset-loader structure source reads"
    );
    assert!(
        !parent.contains("let native_live_host_replay"),
        "typed_error_child_owners.rs should delegate live-host replay/runtime assertions to structure_assertions.rs"
    );
    assert!(
        !parent.contains("let typed_error_children ="),
        "typed_error_child_owners.rs should delegate typed-error child source aggregation to structure_assertions.rs"
    );
    assert_contains_all(
        "typed-error structure assertions parent delegates focused guard children",
        &child,
        &[
            "#[path = \"structure/convergence_mounts.rs\"]",
            "mod convergence_mounts;",
            "#[path = \"structure/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "#[path = \"structure/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
            "convergence_mounts::assert_typed_error_convergence_parents_are_folder_backed",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert_contains_all(
        "typed-error structure assertion children own mount checks and moved guards",
        &child_tree,
        &[
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            "fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
            "fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "super::super::source_inventory::typed_error_children_source",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
            "review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
            "review_f5_ui_input_surrounding_text_error_implements_std_error",
        ],
    );
    for (_, child_path, anchor) in TYPED_ERROR_STRUCTURE_ASSERTION_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "typed-error structure assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error structure assertions child {child_path} should own anchor {anchor}"
        );
    }

    assert_typed_error_child_owners_are_folder_backed();

    for (path, source) in [
        (TYPED_ERROR_STRUCTURE_CHILD, parent),
        (TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD, child),
    ]
    .into_iter()
    .chain(structure_assertion_guard_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
