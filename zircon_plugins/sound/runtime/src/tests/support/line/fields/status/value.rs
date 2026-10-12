use super::super::super::super::super::super::values::capability_status_from_plugin_toml;

pub(super) fn capability_status_status_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::CapabilityStatus {
    capability_status_from_plugin_toml(value)
}
