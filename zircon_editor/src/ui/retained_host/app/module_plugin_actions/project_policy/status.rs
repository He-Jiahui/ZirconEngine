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
    let capacity = target_modes
        .iter()
        .map(|mode| target_mode_status_name(*mode).len())
        .sum::<usize>()
        + target_modes.len().saturating_sub(1) * 2;
    let mut label = String::with_capacity(capacity);
    for (index, mode) in target_modes.iter().enumerate() {
        if index != 0 {
            label.push_str(", ");
        }
        label.push_str(target_mode_status_name(*mode));
    }
    label
}

fn target_mode_status_name(mode: RuntimeTargetMode) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client",
        RuntimeTargetMode::ServerRuntime => "server",
        RuntimeTargetMode::EditorHost => "editor",
    }
}

#[cfg(test)]
#[path = "status/direct_join_tests.rs"]
mod direct_join_tests;
