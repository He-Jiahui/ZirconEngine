use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_structure_subtree_is_child_owned(
    sources: &TypedErrorChildOwnershipSources,
) {
    let structure_assertions_child_tree = typed_error_structure_assertions_child_tree(sources);
    let child_ownership_guard_tree = format!(
        "{}\n{}\n{}",
        sources.child_ownership_child, sources.child, sources.typed_error_sources
    );

    assert_contains_all(
        "code review findings structure child delegates typed-error structure checks",
        &sources.structure_guard_typed_error_child,
        &[
            "fn runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
            "pub(super) fn assert_typed_error_structure_children_are_mounted",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader.rs",
        ],
    );
    assert_contains_all(
        "typed-error structure child delegates focused structure checks",
        &sources.child,
        &[
            "#[path = \"typed_error_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"typed_error_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"typed_error_owners/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"typed_error_owners/structure_assertions.rs\"]",
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
        &child_ownership_guard_tree,
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
        &sources.structure_assertions_child,
        &[
            "#[path = \"structure/convergence_mounts.rs\"]",
            "mod convergence_mounts;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/status_mirrors.rs\"]",
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
            "#[path = \"structure/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "fn runtime_15_typed_error_structure_assertions_are_child_owner",
            "#[path = \"structure/native_plugin_loader.rs\"]",
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
}
