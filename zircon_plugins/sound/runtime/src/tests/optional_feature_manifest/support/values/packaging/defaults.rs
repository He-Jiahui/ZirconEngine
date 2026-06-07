pub(super) fn default_packaging_strategy_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    super::packaging_strategy_list_from_plugin_toml(value)
}
