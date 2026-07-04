use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_live_host_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native live-host typed-error parent mounts focused child owners",
        &sources.native_live_host_parent,
        &[
            "#[path = \"live_host/lifecycle_paths.rs\"]",
            "mod lifecycle_paths;",
            "#[path = \"live_host/replay_and_runtime.rs\"]",
            "mod replay_and_runtime;",
        ],
    );
    assert_eq!(
        sources.native_live_host_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host.rs should only mount child test owners"
    );
}
