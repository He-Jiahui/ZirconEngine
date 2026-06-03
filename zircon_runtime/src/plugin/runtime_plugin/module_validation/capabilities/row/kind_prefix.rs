use crate::plugin::{PluginModuleKind, PluginModuleManifest};

pub(super) fn validate_runtime_plugin_module_capability_kind_prefix(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    match module.kind {
        PluginModuleKind::Runtime if !capability.starts_with("runtime.") => {
            diagnostics.push(format!(
                "{manifest_label} runtime module `{}` capability `{capability}` must start with `runtime.`",
                module.name
            ));
        }
        PluginModuleKind::Editor if !capability.starts_with("editor.") => {
            diagnostics.push(format!(
                "{manifest_label} editor module `{}` capability `{capability}` must start with `editor.`",
                module.name
            ));
        }
        _ => {}
    }
}
