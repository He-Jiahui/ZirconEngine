use super::*;

pub(super) fn assert_typed_error_structure_assertion_checks_are_current() {
    let parent = read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER);
    let typed_error_structure_assertions_child =
        read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER);
    let typed_error_native_structure_child =
        source_trees::typed_error_native_structure_child_tree();
    let typed_error_moved_guard_absence_child =
        source_trees::typed_error_moved_guard_absence_child_tree();
    let typed_error_source_inventory_child =
        source_trees::typed_error_source_inventory_child_tree();
    let structure_child_tree = source_trees::typed_error_structure_assertions_child_tree();

    assert_contains_all(
        "typed-error structure assertions child keeps focused guard mounts",
        &typed_error_structure_assertions_child,
        &[
            "#[path = \"structure/convergence_mounts.rs\"]",
            "mod convergence_mounts;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "structure_assertion_guard_child_sources",
            "structure_assertion_guard_child_source_blob",
        ],
    );
    assert_contains_all(
        "typed-error structure assertions subtree keeps typed-error mount checks and moved-guard delegation",
        &structure_child_tree,
        &[
            "#[path = \"structure/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "#[path = \"structure/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "fn runtime_15_typed_error_structure_assertions_are_child_owner",
            "fn runtime_15_typed_error_structure_assertions_children_are_child_owned",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert_contains_all(
        "typed-error native plugin loader child keeps native mount checks",
        &typed_error_native_structure_child,
        &[
            "fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
            "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
        ],
    );
    assert_contains_all(
        "typed-error moved-guard child keeps review guard ownership checks",
        &typed_error_moved_guard_absence_child,
        &[
            "fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "const PRESERVED_TYPED_ERROR_REVIEW_GUARDS",
            "const PARENT_BACKFLOW_GUARDS",
            "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
            "review_f5_texture_loader_uses_typed_error",
            "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
            "review_f5_asset_authoring_uses_typed_error",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
            "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_contains_all(
        "typed-error source inventory child keeps fine-grained typed-error source paths",
        &typed_error_source_inventory_child,
        &[
            "const TYPED_ERROR_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/zshader.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths/loading.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/registration_replay.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/args_boundary.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/run_boundary.rs",
            "typed_error_review_guard_count",
        ],
    );
    assert!(
        !parent.contains("typed-error structure assertions subtree keeps typed-error mount checks"),
        "typed-error structure assertion guard details should stay in {STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER}"
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned(
) {
    assert_typed_error_structure_assertion_checks_are_current();
}
