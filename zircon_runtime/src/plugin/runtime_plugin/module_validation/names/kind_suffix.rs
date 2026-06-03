use crate::plugin::{PluginModuleKind, PluginModuleManifest};

pub(super) fn validate_runtime_plugin_module_name_kind_suffix(
    manifest_label: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    match module.kind {
        PluginModuleKind::Runtime if !module.name.ends_with(".runtime") => {
            diagnostics.push(format!(
                "{manifest_label} runtime module name `{}` must end with `.runtime`",
                module.name
            ));
        }
        PluginModuleKind::Editor if !module.name.ends_with(".editor") => {
            diagnostics.push(format!(
                "{manifest_label} editor module name `{}` must end with `.editor`",
                module.name
            ));
        }
        _ => {}
    }
}
