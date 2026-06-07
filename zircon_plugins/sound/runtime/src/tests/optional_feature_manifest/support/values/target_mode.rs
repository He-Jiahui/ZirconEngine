mod module;
mod raw;

fn runtime_target_mode_from_plugin_toml(value: String) -> zircon_runtime::RuntimeTargetMode {
    raw::runtime_target_mode_from_plugin_toml(value)
}

fn runtime_target_mode_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::RuntimeTargetMode> {
    super::array::string_array_values(value)
        .into_iter()
        .map(runtime_target_mode_from_plugin_toml)
        .collect()
}

pub(in super::super) fn module_target_mode_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::RuntimeTargetMode> {
    module::module_target_mode_list_from_plugin_toml(value)
}
