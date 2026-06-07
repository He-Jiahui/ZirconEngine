mod raw;

pub(in super::super) fn maturity_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    raw::maturity_from_plugin_toml(value)
}
