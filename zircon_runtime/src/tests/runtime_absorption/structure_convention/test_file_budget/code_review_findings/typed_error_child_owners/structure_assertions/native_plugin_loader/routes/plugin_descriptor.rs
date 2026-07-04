use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_plugin_descriptor_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native plugin descriptor typed-error parent mounts focused child owners",
        &sources.native_plugin_descriptor_parent,
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
        sources
            .native_plugin_descriptor_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs should only mount child test owners"
    );
}
