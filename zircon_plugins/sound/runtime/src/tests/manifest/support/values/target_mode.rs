mod raw;

pub(in super::super) fn runtime_target_mode_from_plugin_toml(
    value: &str,
) -> zircon_runtime::core::framework::platform::RuntimeTargetMode {
    raw::runtime_target_mode_from_plugin_toml(value)
}
