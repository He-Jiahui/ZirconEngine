pub(super) fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

pub(super) fn bool_from_plugin_toml(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("unknown sound boolean value {value}"),
    }
}

pub(super) fn packaging_strategy_from_plugin_toml(
    value: String,
) -> zircon_runtime::plugin::ExportPackagingStrategy {
    match value.as_str() {
        "source_template" => zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
        "library_embed" => zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
        "native_dynamic" => zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        _ => panic!("unknown sound packaging strategy {value}"),
    }
}

pub(super) fn plugin_module_kind_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginModuleKind {
    match value {
        "runtime" => zircon_runtime::plugin::PluginModuleKind::Runtime,
        "editor" => zircon_runtime::plugin::PluginModuleKind::Editor,
        "native" => zircon_runtime::plugin::PluginModuleKind::Native,
        "vm" => zircon_runtime::plugin::PluginModuleKind::Vm,
        _ => panic!("unknown sound module kind {value}"),
    }
}

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
