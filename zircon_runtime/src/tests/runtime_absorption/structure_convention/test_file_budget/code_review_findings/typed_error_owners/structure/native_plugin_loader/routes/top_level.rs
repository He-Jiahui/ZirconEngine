use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_plugin_loader_top_level_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native plugin loader typed-error parent mounts focused child owners",
        &sources.native_plugin_loader_parent,
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
        sources
            .native_plugin_loader_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader.rs should only mount child test owners"
    );
}
