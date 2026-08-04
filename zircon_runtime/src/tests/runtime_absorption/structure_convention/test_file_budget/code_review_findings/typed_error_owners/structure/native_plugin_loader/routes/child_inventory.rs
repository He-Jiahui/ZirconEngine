use super::*;

pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_ROUTE_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_ownership",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD_OWNERSHIP_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_OWNERSHIP_GUARD,
    ),
    (
        "top_level",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_TOP_LEVEL_CHILD,
        "assert_typed_error_native_plugin_loader_top_level_route_is_folder_backed",
    ),
    (
        "abi_surfaces",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_ABI_SURFACES_CHILD,
        "assert_typed_error_native_abi_surfaces_route_is_folder_backed",
    ),
    (
        "plugin_descriptor",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_PLUGIN_DESCRIPTOR_CHILD,
        "assert_typed_error_native_plugin_descriptor_route_is_folder_backed",
    ),
    (
        "live_host",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_LIVE_HOST_CHILD,
        "assert_typed_error_native_live_host_route_is_folder_backed",
    ),
    (
        "lifecycle_paths",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_LIFECYCLE_PATHS_CHILD,
        "assert_typed_error_native_live_host_lifecycle_paths_route_is_folder_backed",
    ),
    (
        "replay_runtime",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_REPLAY_RUNTIME_CHILD,
        "assert_typed_error_native_live_host_replay_runtime_route_is_folder_backed",
    ),
    (
        "manifest_sources",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_MANIFEST_SOURCES_CHILD,
        "assert_typed_error_native_manifest_sources_route_is_folder_backed",
    ),
];

pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "child_inventory",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD_INVENTORY_CHILD,
        "TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_CHILDREN",
    ),
    (
        "metadata",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_METADATA_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_SLICE,
    ),
    (
        "source_helper_ownership",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_OWNERSHIP_CHILD,
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_OWNERSHIP_GUARD,
    ),
    (
        "sources",
        TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCES_CHILD,
        "pub(in super::super) fn typed_error_native_plugin_loader_route_child_sources",
    ),
];
