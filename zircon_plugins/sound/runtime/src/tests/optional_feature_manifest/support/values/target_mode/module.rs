pub(super) fn module_target_mode_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::RuntimeTargetMode> {
    super::runtime_target_mode_list_from_plugin_toml(value)
}
