use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_live_host_lifecycle_paths_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native live-host lifecycle-paths typed-error parent mounts focused child owners",
        &sources.native_live_host_lifecycle_paths_parent,
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
        sources
            .native_live_host_lifecycle_paths_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs should only mount child test owners"
    );
}
