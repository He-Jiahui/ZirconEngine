use super::super::values::string_array_values;
use super::state::PendingOptionManifest;

pub(super) fn option_manifests_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    let mut options = Vec::new();
    let mut pending = PendingOptionManifest::default();
    let mut inside_option = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[options]]" {
            pending.push_into(&mut options);
            inside_option = true;
            continue;
        }
        if line.starts_with("[[") {
            pending.push_into(&mut options);
            inside_option = false;
        }
        if !inside_option {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("key = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            pending.key = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("display_name = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            pending.display_name = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("value_type = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            pending.value_type = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("default_value = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            pending.default_value = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("enum_values = [")
            .and_then(|value| value.strip_suffix(']'))
        {
            pending.enum_values = string_array_values(value);
            continue;
        }
        if let Some(value) = line
            .strip_prefix("required_capability = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            pending.required_capability = Some(value.to_string());
        }
    }
    pending.push_into(&mut options);
    options
}
