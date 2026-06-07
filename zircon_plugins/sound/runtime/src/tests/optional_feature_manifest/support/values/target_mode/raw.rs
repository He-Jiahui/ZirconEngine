pub(super) fn runtime_target_mode_from_plugin_toml(
    value: String,
) -> zircon_runtime::RuntimeTargetMode {
    match value.as_str() {
        "client_runtime" => zircon_runtime::RuntimeTargetMode::ClientRuntime,
        "editor_host" => zircon_runtime::RuntimeTargetMode::EditorHost,
        "server_runtime" => zircon_runtime::RuntimeTargetMode::ServerRuntime,
        _ => panic!("unknown sound module target mode {value}"),
    }
}
