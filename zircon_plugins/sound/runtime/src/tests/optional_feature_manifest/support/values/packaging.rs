mod defaults;
mod raw;

fn packaging_strategy_from_plugin_toml(
    value: String,
) -> zircon_runtime::plugin::ExportPackagingStrategy {
    raw::packaging_strategy_from_plugin_toml(value)
}

fn packaging_strategy_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    super::array::string_array_values(value)
        .into_iter()
        .map(packaging_strategy_from_plugin_toml)
        .collect()
}

pub(in super::super) fn default_packaging_strategy_list_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    defaults::default_packaging_strategy_list_from_plugin_toml(value)
}
