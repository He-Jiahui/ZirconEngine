//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
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
