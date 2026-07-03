use super::super::super::super::*;

const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs";
const TYPED_ERROR_NATIVE_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs";
const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) fn assert_typed_error_native_plugin_loader_children_are_folder_backed() {
    let native_plugin_loader_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
    );
    let native_abi_surfaces_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
    );
    let native_plugin_descriptor_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
    );
    let native_live_host_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
    );
    let native_live_host_lifecycle_paths_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
    );
    let native_live_host_replay_and_runtime_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
    );
    let native_manifest_sources_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
    );

    assert_contains_all(
        "native plugin loader typed-error parent mounts focused child owners",
        &native_plugin_loader_parent,
        &[
            "#[path = \"native_plugin_loader/abi_surfaces.rs\"]",
            "mod abi_surfaces;",
            "#[path = \"native_plugin_loader/bridge_lifecycle.rs\"]",
            "mod bridge_lifecycle;",
            "#[path = \"native_plugin_loader/diagnostics.rs\"]",
            "mod diagnostics;",
            "#[path = \"native_plugin_loader/live_host.rs\"]",
            "mod live_host;",
            "#[path = \"native_plugin_loader/manifest_sources.rs\"]",
            "mod manifest_sources;",
        ],
    );
    assert_eq!(
        native_plugin_loader_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader.rs should only mount child test owners"
    );
    assert_contains_all(
        "native ABI surfaces typed-error parent mounts focused child owners",
        &native_abi_surfaces_parent,
        &[
            "#[path = \"abi_surfaces/behavior_bridge.rs\"]",
            "mod behavior_bridge;",
            "#[path = \"abi_surfaces/plugin_descriptor.rs\"]",
            "mod plugin_descriptor;",
            "#[path = \"abi_surfaces/host_adapter.rs\"]",
            "mod host_adapter;",
        ],
    );
    assert_eq!(
        native_abi_surfaces_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader/abi_surfaces.rs should only mount child test owners"
    );
    assert_contains_all(
        "native plugin descriptor typed-error parent mounts focused child owners",
        &native_plugin_descriptor_parent,
        &[
            "#[path = \"plugin_descriptor/string_helpers.rs\"]",
            "mod string_helpers;",
            "#[path = \"plugin_descriptor/descriptor_abi.rs\"]",
            "mod descriptor_abi;",
            "#[path = \"plugin_descriptor/entry_abi.rs\"]",
            "mod entry_abi;",
        ],
    );
    assert_eq!(
        native_plugin_descriptor_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs should only mount child test owners"
    );
    assert_contains_all(
        "native live-host typed-error parent mounts focused child owners",
        &native_live_host_parent,
        &[
            "#[path = \"live_host/lifecycle_paths.rs\"]",
            "mod lifecycle_paths;",
            "#[path = \"live_host/replay_and_runtime.rs\"]",
            "mod replay_and_runtime;",
        ],
    );
    assert_eq!(
        native_live_host_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host.rs should only mount child test owners"
    );
    assert_contains_all(
        "native live-host lifecycle-paths typed-error parent mounts focused child owners",
        &native_live_host_lifecycle_paths_parent,
        &[
            "#[path = \"lifecycle_paths/hot_reload.rs\"]",
            "mod hot_reload;",
            "#[path = \"lifecycle_paths/lifecycle.rs\"]",
            "mod lifecycle;",
            "#[path = \"lifecycle_paths/loading.rs\"]",
            "mod loading;",
        ],
    );
    assert_eq!(
        native_live_host_lifecycle_paths_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs should only mount child test owners"
    );
    assert_contains_all(
        "native live-host replay-runtime typed-error parent mounts focused child owners",
        &native_live_host_replay_and_runtime_parent,
        &[
            "#[path = \"replay_and_runtime/bridge_methods.rs\"]",
            "mod bridge_methods;",
            "#[path = \"replay_and_runtime/registration_replay.rs\"]",
            "mod registration_replay;",
            "#[path = \"replay_and_runtime/runtime_behavior.rs\"]",
            "mod runtime_behavior;",
        ],
    );
    assert_eq!(
        native_live_host_replay_and_runtime_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs should only mount child test owners"
    );
    assert_contains_all(
        "native manifest sources typed-error parent mounts focused child owners",
        &native_manifest_sources_parent,
        &[
            "#[path = \"manifest_sources/compat_registration.rs\"]",
            "mod compat_registration;",
            "#[path = \"manifest_sources/collection_candidate.rs\"]",
            "mod collection_candidate;",
        ],
    );
    assert_eq!(
        native_manifest_sources_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader/manifest_sources.rs should only mount child test owners"
    );
}

#[test]
fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    let child = read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD);

    assert_contains_all(
        "typed-error structure assertions delegates native plugin loader checks",
        &parent,
        &[
            "#[path = \"structure_assertions/native_plugin_loader.rs\"]",
            "mod native_plugin_loader;",
            "native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed",
        ],
    );
    assert!(
        !parent.contains("let native_plugin_loader_parent = read_runtime_src"),
        "structure_assertions.rs should delegate native plugin loader source reads"
    );
    assert!(
        !parent.contains("native_live_host_replay_and_runtime_parent"),
        "structure_assertions.rs should delegate native live-host replay/runtime assertions"
    );
    assert_contains_all(
        "typed-error native plugin loader child owns native mount checks",
        &child,
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

    assert_typed_error_native_plugin_loader_children_are_folder_backed();

    for (path, source) in [
        (TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD, parent.as_str()),
        (TYPED_ERROR_NATIVE_STRUCTURE_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
