use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::ExportPackagingStrategy;

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_primary_action(
    plugin_id: &str,
    enabled: bool,
    required: bool,
) -> (String, String) {
    if required {
        return ("Required".to_string(), String::new());
    }

    if enabled {
        (
            "Disable".to_string(),
            module_plugin_action_id("workbench.plugin.disable", plugin_id),
        )
    } else {
        (
            "Enable".to_string(),
            module_plugin_action_id("workbench.plugin.enable", plugin_id),
        )
    }
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_action_id(
    prefix: &str,
    plugin_id: &str,
) -> String {
    format!("{prefix}.{plugin_id}")
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn target_mode_label(
    mode: &RuntimeTargetMode,
) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client",
        RuntimeTargetMode::ServerRuntime => "server",
        RuntimeTargetMode::EditorHost => "editor",
    }
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn packaging_label(
    strategy: ExportPackagingStrategy,
) -> &'static str {
    match strategy {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}
