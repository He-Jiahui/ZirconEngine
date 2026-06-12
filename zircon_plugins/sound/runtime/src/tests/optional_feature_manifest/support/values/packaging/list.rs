use super::raw::packaging_strategy_from_plugin_toml;

pub(super) fn packaging_strategy_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    super::super::array::string_list_from_plugin_toml(value)
        .into_iter()
        .map(packaging_strategy_from_plugin_toml)
        .collect()
}
