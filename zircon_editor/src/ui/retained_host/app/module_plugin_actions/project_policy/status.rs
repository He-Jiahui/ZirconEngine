use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

pub(in crate::ui::retained_host::app::module_plugin_actions) fn packaging_status_label(
    packaging: ExportPackagingStrategy,
) -> &'static str {
    match packaging {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

pub(in crate::ui::retained_host::app::module_plugin_actions) fn target_modes_status_label(
    target_modes: &[RuntimeTargetMode],
) -> String {
    if target_modes.is_empty() {
        return "all".to_string();
    }
    target_modes
        .iter()
        .map(|mode| match mode {
            RuntimeTargetMode::ClientRuntime => "client",
            RuntimeTargetMode::ServerRuntime => "server",
            RuntimeTargetMode::EditorHost => "editor",
        })
        .collect::<Vec<_>>()
        .join(", ")
}
