use super::super::super::values::bool_from_plugin_toml;

pub(in super::super) fn parse_optional_feature_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    if let Some(value) = line
        .strip_prefix("plugin_id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *plugin_id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("capability = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *capability = Some(value.to_string());
        return;
    }
    if let Some(value) = line.strip_prefix("primary = ") {
        *primary = Some(bool_from_plugin_toml(value));
    }
}
