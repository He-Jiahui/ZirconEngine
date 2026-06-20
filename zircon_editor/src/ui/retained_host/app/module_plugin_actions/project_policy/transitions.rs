use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::ExportPackagingStrategy;

pub(in crate::ui::retained_host::app::module_plugin_actions) fn next_packaging(
    packaging: ExportPackagingStrategy,
) -> ExportPackagingStrategy {
    match packaging {
        ExportPackagingStrategy::LibraryEmbed => ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::NativeDynamic => ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::SourceTemplate => ExportPackagingStrategy::LibraryEmbed,
    }
}

pub(in crate::ui::retained_host::app::module_plugin_actions) fn next_target_modes(
    target_modes: &[RuntimeTargetMode],
) -> Vec<RuntimeTargetMode> {
    match target_modes {
        [] => vec![RuntimeTargetMode::ClientRuntime],
        [RuntimeTargetMode::ClientRuntime] => vec![RuntimeTargetMode::ServerRuntime],
        [RuntimeTargetMode::ServerRuntime] => vec![RuntimeTargetMode::EditorHost],
        [RuntimeTargetMode::EditorHost] => {
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]
        }
        [RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::EditorHost] => Vec::new(),
        _ => Vec::new(),
    }
}
