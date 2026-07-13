use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_abi_surfaces_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native ABI surfaces typed-error parent mounts focused child owners",
        &sources.native_abi_surfaces_parent,
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
        sources
            .native_abi_surfaces_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/abi_surfaces.rs should only mount child test owners"
    );
}
