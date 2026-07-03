use super::*;

#[test]
fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PARENT);
    let child = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let child_ownership_child = read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD);
    let structure_assertions_child = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let convergence_mounts_child = read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD);
    let delegation_child = read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD);
    let child_ownership_structure_child =
        read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD);
    let status_mirrors_child = read_runtime_src(TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD);
    let moved_guard_absence_child =
        read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
    let moved_guard_absence_preserved_guards_child = read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/preserved_guards.rs");
    let moved_guard_absence_parent_backflow_child = read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/parent_backflow.rs");
    let moved_guard_absence_path_anchors_child = read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/path_anchors.rs");
    let moved_guard_absence_budgets_child = read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/budgets.rs");
    let moved_guard_absence_status_mirrors_child = read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/status_mirrors.rs");
    let moved_guard_absence_child_tree = [
        moved_guard_absence_child.as_str(),
        moved_guard_absence_preserved_guards_child.as_str(),
        moved_guard_absence_parent_backflow_child.as_str(),
        moved_guard_absence_path_anchors_child.as_str(),
        moved_guard_absence_budgets_child.as_str(),
        moved_guard_absence_status_mirrors_child.as_str(),
    ]
    .join("\n");
    let native_plugin_loader_child = read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD);
    let structure_guard_typed_error_child = read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD);
    let typed_error_sources = typed_error_children_source();
    let structure_assertions_child_tree = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        structure_assertions_child,
        convergence_mounts_child,
        delegation_child,
        child_ownership_structure_child,
        status_mirrors_child,
        moved_guard_absence_child,
        native_plugin_loader_child
    );

    assert_contains_all(
        "code review findings structure guard parent mounts typed-error child owner",
        &parent,
        &[
            "#[path = \"code_review_findings/typed_error_child_owners.rs\"]",
            "mod typed_error_child_owners;",
        ],
    );
    assert_contains_all(
        "code review findings structure child delegates typed-error structure checks",
        &structure_guard_typed_error_child,
        &[
            "fn runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
            "pub(super) fn assert_typed_error_structure_children_are_mounted",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs",
        ],
    );
    assert_contains_all(
        "typed-error structure child delegates focused structure checks",
        &child,
        &[
            "#[path = \"typed_error_child_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"typed_error_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"typed_error_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"typed_error_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "structure_assertions::assert_typed_error_child_owners_are_folder_backed",
            "source_inventory::typed_error_children_source",
            "source_inventory::assert_typed_error_line_budgets",
            "source_inventory::typed_error_review_guard_count",
            "status_docs::assert_typed_error_status_docs_are_synced",
        ],
    );
    assert_contains_all(
        "typed-error top-level child ownership keeps historical guard",
        &child_ownership_child,
        &[
            GUARD,
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
            "typed_error_review_guard_count",
            "assert_typed_error_status_docs_are_synced",
        ],
    );
    assert_contains_all(
        "typed-error structure assertions child mounts focused folder-backed guard children",
        &structure_assertions_child,
        &[
            "#[path = \"structure_assertions/convergence_mounts.rs\"]",
            "mod convergence_mounts;",
            "#[path = \"structure_assertions/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure_assertions/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure_assertions/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
            "convergence_mounts::assert_typed_error_convergence_parents_are_folder_backed",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert_contains_all(
        "typed-error structure assertion subtree owns typed-error mount checks and moved guards",
        &structure_assertions_child_tree,
        &[
            "#[path = \"structure_assertions/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "fn runtime_15_typed_error_structure_assertions_are_child_owner",
            "#[path = \"structure_assertions/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "fn runtime_15_typed_error_structure_assertions_children_are_child_owned",
            "fn runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            "moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert_contains_all(
        "typed-error native plugin loader child owns native structure checks",
        &native_plugin_loader_child,
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
        "typed-error moved-guard child owns review guard preservation checks",
        &moved_guard_absence_child_tree,
        &[
            "fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "super::super::source_inventory::typed_error_children_source",
            "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/descriptor_abi.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/entry_abi.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surface_effects.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surrounding_text.rs",
            "review_f5_texture_loader_uses_typed_error",
            "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
            "review_f5_asset_authoring_uses_typed_error",
            "review_f5_navigation_asset_uses_typed_error",
            "review_f5_font_asset_uses_typed_error_source",
            "review_f5_sound_asset_uses_typed_error",
            "review_f5_zshader_v2_replaces_user_shader_definitions",
            "review_f5_asset_meta_uses_typed_error",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
            "review_f5_native_live_host_loading_uses_typed_error",
            "review_f5_native_live_host_hot_reload_uses_typed_error",
            "review_f5_native_live_host_registration_replay_uses_typed_error",
            "review_f5_native_live_host_bridge_methods_use_typed_error",
            "review_f5_native_live_host_runtime_behavior_uses_typed_error",
            "review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
            "review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
            "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
            "review_f5_ui_input_surrounding_text_error_implements_std_error",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );

    assert_typed_error_child_owners_are_folder_backed();
    assert_typed_error_line_budgets();
    assert_contains_all(
        "typed-error source helper preserves review guard anchors",
        &typed_error_sources,
        &[
            "review_f5_texture_loader_uses_typed_error",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_eq!(
        typed_error_review_guard_count(),
        47,
        "typed-error child owners should preserve all current F5/F6/F7 review guards"
    );

    for (path, source) in [
        (STRUCTURE_GUARD_PARENT, parent.as_str()),
        (
            STRUCTURE_GUARD_TYPED_ERROR_CHILD,
            structure_guard_typed_error_child.as_str(),
        ),
        (TYPED_ERROR_STRUCTURE_CHILD, child.as_str()),
        (
            TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
            child_ownership_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
            structure_assertions_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
            convergence_mounts_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
            delegation_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
            child_ownership_structure_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
            status_mirrors_child.as_str(),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
            native_plugin_loader_child.as_str(),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
            moved_guard_absence_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_typed_error_status_docs_are_synced();
}
