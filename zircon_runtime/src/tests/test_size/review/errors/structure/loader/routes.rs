//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::*;
use super::*;

#[path = "../../../../../review/errors/structure/loader/routes/abi_surfaces.rs"]
mod abi_surfaces;
#[path = "../../../../../review/errors/loader/routes/child_inventory.rs"]
mod child_inventory;
#[path = "../../../../../review/errors/loader/routes/child_ownership.rs"]
mod child_ownership;
#[path = "../../../../../review/errors/loader/routes/lifecycle_paths.rs"]
mod lifecycle_paths;
#[path = "../../../../../review/errors/structure/loader/routes/live_host.rs"]
mod live_host;
#[path = "../../../../../review/errors/loader/routes/manifest_sources.rs"]
mod manifest_sources;
#[path = "../../../../../review/errors/structure/loader/routes/metadata.rs"]
mod metadata;
#[path = "../../../../../review/errors/loader/routes/plugin_descriptor.rs"]
mod plugin_descriptor;
#[path = "../../../../../review/errors/structure/loader/routes/replay_runtime.rs"]
mod replay_runtime;
#[path = "../../../../../review/errors/loader/routes/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "../../../../../review/errors/structure/loader/routes/sources.rs"]
mod sources;
#[path = "../../../../../review/errors/structure/loader/routes/top_level.rs"]
mod top_level;

pub(super) use child_inventory::*;
pub(super) use metadata::*;
pub(super) use sources::*;

pub(super) fn assert_typed_error_native_plugin_loader_routes_are_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    top_level::assert_typed_error_native_plugin_loader_top_level_route_is_folder_backed(sources);
    abi_surfaces::assert_typed_error_native_abi_surfaces_route_is_folder_backed(sources);
    plugin_descriptor::assert_typed_error_native_plugin_descriptor_route_is_folder_backed(sources);
    live_host::assert_typed_error_native_live_host_route_is_folder_backed(sources);
    lifecycle_paths::assert_typed_error_native_live_host_lifecycle_paths_route_is_folder_backed(
        sources,
    );
    replay_runtime::assert_typed_error_native_live_host_replay_runtime_route_is_folder_backed(
        sources,
    );
    manifest_sources::assert_typed_error_native_manifest_sources_route_is_folder_backed(sources);
}
