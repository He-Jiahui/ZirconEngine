use super::super::values::maturity_from_plugin_toml;

pub(super) fn static_maturity_from_plugin_toml(
    manifest: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    for line in manifest.lines().map(str::trim) {
        let Some(value) = line
            .strip_prefix("maturity = \"")
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        return maturity_from_plugin_toml(value);
    }
    panic!("sound plugin.toml should declare maturity")
}
