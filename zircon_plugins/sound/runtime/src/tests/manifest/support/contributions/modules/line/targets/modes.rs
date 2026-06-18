use super::super::super::super::super::values::{
    runtime_target_mode_from_plugin_toml, string_array_values,
};

pub(super) fn module_target_modes_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::builtin::RuntimeTargetMode> {
    string_array_values(value)
        .into_iter()
        .map(|mode| runtime_target_mode_from_plugin_toml(&mode))
        .collect()
}
