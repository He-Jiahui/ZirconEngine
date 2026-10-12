use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_runtime_extensions_for_target;
use super::super::RuntimePluginRegistrationReport;

// selection 提供已排序的登记索引；这里只合并所选包，并把当前 target 继续传给模块贡献过滤。
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_selected_runtime_extensions(
    registrations: &[RuntimePluginRegistrationReport],
    selected_registration_indices: &[usize],
    target: RuntimeTargetMode,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for index in selected_registration_indices {
        merge_runtime_extensions_for_target(
            &registrations[*index],
            target,
            registry,
            diagnostics,
            fatal_diagnostics,
        );
    }
}
