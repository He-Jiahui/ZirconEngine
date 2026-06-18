pub(in super::super::super::super) fn module_target_mode_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::builtin::RuntimeTargetMode> {
    super::super::list::runtime_target_mode_list_from_plugin_toml(value)
}
