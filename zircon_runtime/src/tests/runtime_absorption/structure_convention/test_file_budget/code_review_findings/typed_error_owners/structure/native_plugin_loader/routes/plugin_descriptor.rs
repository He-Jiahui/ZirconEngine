//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
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
