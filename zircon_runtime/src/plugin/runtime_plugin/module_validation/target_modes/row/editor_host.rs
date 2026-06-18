use crate::builtin::RuntimeTargetMode;
use crate::plugin::{PluginModuleKind, PluginModuleManifest};

pub(super) fn validate_runtime_plugin_module_editor_host_target_mode(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_mode: RuntimeTargetMode,
    diagnostics: &mut Vec<String>,
) {
    if module.kind == PluginModuleKind::Editor && target_mode != RuntimeTargetMode::EditorHost {
        diagnostics.push(format!(
            "{manifest_label} editor module `{}` target mode {target_mode:?} must be EditorHost",
            module.name
        ));
    }
}
