//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
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
