mod raw;

pub(in super::super) fn capability_status_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::CapabilityStatus {
    raw::capability_status_from_plugin_toml(value)
}
