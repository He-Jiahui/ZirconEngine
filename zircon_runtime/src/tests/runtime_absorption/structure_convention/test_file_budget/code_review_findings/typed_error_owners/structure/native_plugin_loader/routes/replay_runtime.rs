//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_native_live_host_replay_runtime_route_is_folder_backed(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    assert_contains_all(
        "native live-host replay-runtime typed-error parent mounts focused child owners",
        &sources.native_live_host_replay_and_runtime_parent,
        &[
            "#[path = \"replay_and_runtime/bridge_methods.rs\"]",
            "mod bridge_methods;",
            "#[path = \"replay_and_runtime/registration_replay.rs\"]",
            "mod registration_replay;",
            "#[path = \"replay_and_runtime/runtime_behavior.rs\"]",
            "mod runtime_behavior;",
        ],
    );
    assert_eq!(
        sources
            .native_live_host_replay_and_runtime_parent
            .matches("#[test]")
            .count(),
        0,
        "typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs should only mount child test owners"
    );
}
