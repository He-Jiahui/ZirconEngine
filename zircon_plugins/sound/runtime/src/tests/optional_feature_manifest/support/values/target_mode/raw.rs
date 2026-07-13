pub(in super::super) fn runtime_target_mode_from_plugin_toml(
    value: String,
) -> zircon_runtime::core::framework::platform::RuntimeTargetMode {
    match value.as_str() {
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
