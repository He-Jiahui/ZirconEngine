use super::super::super::values::{
    plugin_module_kind_from_plugin_toml, runtime_target_mode_from_plugin_toml, string_array_values,
};

pub(super) fn parse_module_contribution_line(
    line: &str,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    if let Some(value) = line
        .strip_prefix("name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("kind = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *kind = Some(plugin_module_kind_from_plugin_toml(value));
        return;
    }
    if let Some(value) = line
        .strip_prefix("crate_name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *crate_name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("target_modes = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        *target_modes = string_array_values(value)
            .into_iter()
            .map(|mode| runtime_target_mode_from_plugin_toml(&mode))
            .collect();
        return;
    }
    if let Some(value) = line
        .strip_prefix("capabilities = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        *capabilities = string_array_values(value);
    }
}
