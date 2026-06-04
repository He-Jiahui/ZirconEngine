use super::super::super::types::PendingOptionalFeatureManifest;
use super::super::super::values::{
    bool_from_plugin_toml, packaging_strategy_from_plugin_toml, string_array_values,
};

pub(in super::super) fn parse_optional_feature_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) {
    if let Some(value) = line
        .strip_prefix("id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("display_name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.display_name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("owner_plugin_id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.owner_plugin_id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("capabilities = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        feature.capabilities = string_array_values(value);
        return;
    }
    if let Some(value) = line
        .strip_prefix("default_packaging = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        feature.default_packaging = string_array_values(value)
            .into_iter()
            .map(packaging_strategy_from_plugin_toml)
            .collect();
        return;
    }
    if let Some(value) = line.strip_prefix("enabled_by_default = ") {
        feature.enabled_by_default = Some(bool_from_plugin_toml(value));
    }
}
