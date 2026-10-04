// 对齐静态模块目标与运行时平台模式，避免不同目标的模块在导出比较中混淆。
pub(super) fn runtime_target_mode_from_plugin_toml(
    value: &str,
) -> zircon_runtime::core::framework::platform::RuntimeTargetMode {
    match value {
        "client_runtime" => {
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        }
        "editor_host" => zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        "server_runtime" => {
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime
        }
        _ => panic!("unknown sound module target mode {value}"),
    }
}
