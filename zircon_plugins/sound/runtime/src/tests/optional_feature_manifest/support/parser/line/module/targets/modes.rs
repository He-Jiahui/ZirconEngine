use super::super::super::super::super::values::module_target_mode_list_from_plugin_toml;

pub(super) fn module_target_modes_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::RuntimeTargetMode> {
    module_target_mode_list_from_plugin_toml(value)
}
