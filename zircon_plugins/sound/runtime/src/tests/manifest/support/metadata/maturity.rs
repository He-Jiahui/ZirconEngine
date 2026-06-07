mod entry;
mod field;
mod required;
mod value;

pub(super) fn static_maturity_from_plugin_toml(
    manifest: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    entry::static_maturity_from_plugin_toml(manifest)
}
