use super::super::super::super::super::super::values::default_packaging_strategy_list_from_plugin_toml;

pub(super) fn default_packaging_from_plugin_toml(
    value: &str,
) -> Vec<zircon_runtime::core::framework::project::ExportPackagingStrategy> {
    default_packaging_strategy_list_from_plugin_toml(value)
}
