use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_structure_assertions_children_are_child_owned() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let convergence_mounts_child = [
        read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD),
        typed_error_convergence_mount_nested_child_tree(),
    ]
    .join("\n");
    let child_tree = structure_assertion_guard_child_source_blob();

    for child_owned_guard in [
        "let asset_loaders_parent = read_runtime_src",
        "let asset_records_parent = read_runtime_src",
        "let native_plugin_loader_parent = read_runtime_src",
        "let typed_error_children = super::super::source_inventory",
        "const PARENT_BACKFLOW_GUARDS",
    ] {
        assert!(
            !parent.contains(child_owned_guard),
            "typed-error structure assertion guard `{child_owned_guard}` should stay in a focused child"
        );
    }
    assert_contains_all(
        "typed-error structure assertions parent mounts focused guard children",
        &parent,
        &[
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/convergence_mounts.rs\"]",
            "mod convergence_mounts;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"structure/moved_guard_absence.rs\"]",
            "mod moved_guard_absence;",
            "#[path = \"structure/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "structure_assertion_guard_child_sources",
            "structure_assertion_guard_child_source_blob",
        ],
    );
    assert_contains_all(
        "typed-error convergence mounts child owns parent mount assertions",
        &convergence_mounts_child,
        &[
            "pub(super) fn assert_typed_error_convergence_parents_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            "mod animation_binary;",
            "mod zshader;",
            "mod typed_mutation_surface;",
            "mod gameplay_scene;",
            "mod args_boundary;",
            "mod surrounding_text;",
        ],
    );
    assert_contains_all(
        "typed-error structure assertion guard children own delegated assertions",
        &child_tree,
        &[
            "runtime_15_typed_error_structure_assertions_are_child_owner",
            "runtime_15_typed_error_structure_assertions_children_are_child_owned",
            "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
            "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current",
        ],
    );

    assert_typed_error_child_owners_are_folder_backed();

    for (path, source) in [(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD, parent)]
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

fn typed_error_convergence_mount_nested_child_tree() -> String {
    [
        "top_level",
        "asset_parents",
        "runtime_parents",
        "budgets",
        "status_mirrors",
        "root_paths",
        "root_statuses",
        "root_child_rows",
        "root_sources",
        "root_inventory",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}
