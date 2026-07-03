use super::super::super::*;

const TYPED_ERROR_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
const TYPED_ERROR_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs";
const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

const TYPED_ERROR_SOURCE_PATHS: &[&str] = &[
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/animation_binary.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/artifact_importer.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/mesh_obj.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/authoring.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/font.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/meta.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/navigation.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/sound.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/zshader.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/dynamic_api.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/export_cli.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/behavior_bridge.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/descriptor_abi.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/entry_abi.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/host_adapter.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/bridge_lifecycle.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/diagnostics.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths/hot_reload.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths/lifecycle.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths/loading.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/bridge_methods.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/registration_replay.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/runtime_behavior.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources/compat_registration.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources/collection_candidate.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/typed_mutation_surface.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/fixed_mutation.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/dynamic_components.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/gameplay_scene.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/plugin_management.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/host_reflection_docs.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/args_boundary.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/run_boundary.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_asset_documents.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surface_effects.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surrounding_text.rs",
    "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_template_resource.rs",
];

fn typed_error_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_SOURCE_PATHS
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_children_source() -> String {
    let mut children = String::new();
    for (_, source) in typed_error_sources() {
        children.push_str(&source);
        children.push('\n');
    }
    children
}

pub(super) fn assert_typed_error_line_budgets() {
    for (path, source) in typed_error_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the typed-error child-owner budget; got {line_count} lines"
        );
    }
}

pub(super) fn typed_error_review_guard_count() -> usize {
    typed_error_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}

#[test]
fn runtime_15_typed_error_source_inventory_is_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let child = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD);

    assert_contains_all(
        "typed-error structure guard delegates source inventory to child owner",
        &parent,
        &[
            "#[path = \"typed_error_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "source_inventory::typed_error_children_source",
            "source_inventory::assert_typed_error_line_budgets",
            "source_inventory::typed_error_review_guard_count",
        ],
    );
    assert!(
        !parent.contains("const TYPED_ERROR_SOURCE_PATHS"),
        "typed_error_child_owners.rs should not retain the typed-error source inventory"
    );
    assert!(
        !parent.contains("fn typed_error_sources()"),
        "typed_error_child_owners.rs should delegate typed-error source reads to source_inventory.rs"
    );
    assert_contains_all(
        "typed-error source inventory child owns source paths and count helpers",
        &child,
        &[
            "const TYPED_ERROR_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/runtime_behavior.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/args_boundary.rs",
            "pub(super) fn typed_error_children_source",
            "pub(super) fn assert_typed_error_line_budgets",
            "pub(super) fn typed_error_review_guard_count",
        ],
    );
    assert_eq!(
        typed_error_review_guard_count(),
        47,
        "typed-error source inventory should preserve all current F5/F6/F7 review guards"
    );

    for (path, source) in [
        (TYPED_ERROR_STRUCTURE_CHILD, parent.as_str()),
        (TYPED_ERROR_SOURCE_INVENTORY_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
