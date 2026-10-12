use super::super::super::values::maturity_from_plugin_toml;

pub(super) fn maturity_from_static_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    maturity_from_plugin_toml(value)
}
