use super::{field, required, value};

pub(super) fn static_maturity_from_plugin_toml(
    manifest: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    for line in manifest.lines().map(str::trim) {
        let Some(value) = field::maturity_value(line) else {
            continue;
        };
        return value::maturity_from_static_plugin_toml(value);
    }
    required::missing_static_maturity()
}
