pub(super) fn runtime_target_mode_from_plugin_toml(
    value: &str,
) -> zircon_runtime::builtin::RuntimeTargetMode {
    match value {
        "client_runtime" => zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        "editor_host" => zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        "server_runtime" => zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        _ => panic!("unknown sound module target mode {value}"),
    }
}
