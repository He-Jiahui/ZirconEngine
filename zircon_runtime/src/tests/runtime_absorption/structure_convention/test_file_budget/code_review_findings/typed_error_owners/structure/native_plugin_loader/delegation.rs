use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_native_plugin_loader_structure_is_child_owner(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    let child_tree = typed_error_native_plugin_loader_child_source_blob();
    let native_structure_tree = typed_error_native_plugin_loader_structure_source_tree(sources);

    assert_contains_all(
        "typed-error structure assertions delegates native plugin loader checks",
        &sources.structure_assertions_parent,
        &[
            "#[path = \"structure/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert!(
        !sources
            .structure_assertions_parent
            .contains("let native_plugin_loader_parent = read_runtime_src"),
        "structure_assertions.rs should delegate native plugin loader source reads"
    );
    assert!(
        !sources
            .structure_assertions_parent
            .contains("native_live_host_replay_and_runtime_parent"),
        "structure_assertions.rs should delegate native live-host replay/runtime assertions"
    );
    assert_contains_all(
        "typed-error native plugin loader child owns native mount checks",
        &native_structure_tree,
        &[
            "pub(super) fn assert_typed_error_native_plugin_loader_children_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
            "mod abi_surfaces;",
            "mod plugin_descriptor;",
            "mod lifecycle_paths;",
            "mod replay_and_runtime;",
            "mod manifest_sources;",
        ],
    );
    for (_, child_path, anchor) in TYPED_ERROR_NATIVE_STRUCTURE_GUARD_CHILDREN {
        assert!(
            native_structure_tree.contains(child_path),
            "typed-error native plugin loader source tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error native plugin loader child {child_path} should own anchor {anchor}"
        );
    }
}

#[test]
fn runtime_15_typed_error_native_plugin_loader_structure_guard_is_folder_backed() {
    let sources = typed_error_native_plugin_loader_sources();
    assert_typed_error_native_plugin_loader_structure_is_child_owner(&sources);
    super::routes::assert_typed_error_native_plugin_loader_routes_are_folder_backed(&sources);
}
