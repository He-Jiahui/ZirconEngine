use super::raw::runtime_target_mode_from_plugin_toml;

pub(super) fn runtime_target_mode_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::builtin::RuntimeTargetMode> {
    super::super::array::string_list_from_plugin_toml(value)
        .into_iter()
        .map(runtime_target_mode_from_plugin_toml)
        .collect()
}
