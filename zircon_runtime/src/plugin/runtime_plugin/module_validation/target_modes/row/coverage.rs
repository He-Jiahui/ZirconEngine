use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_target_mode_coverage(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_mode: RuntimeTargetMode,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    if let Some((coverage_label, covered_targets)) = target_coverage {
        if !covered_targets.contains(&target_mode) {
            diagnostics.push(format!(
                "{manifest_label} module `{}` target mode {target_mode:?} must be covered by {coverage_label}",
                module.name
            ));
        }
    }
}
