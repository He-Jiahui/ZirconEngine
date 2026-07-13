use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_manifest_sources_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native manifest sources typed-error parent mounts focused child owners",
        &sources.native_manifest_sources_parent,
        &[
            "#[path = \"manifest_sources/compat_registration.rs\"]",
            "mod compat_registration;",
            "#[path = \"manifest_sources/collection_candidate.rs\"]",
            "mod collection_candidate;",
        ],
    );
    assert_eq!(
        sources
            .native_manifest_sources_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/manifest_sources.rs should only mount child test owners"
    );
}
