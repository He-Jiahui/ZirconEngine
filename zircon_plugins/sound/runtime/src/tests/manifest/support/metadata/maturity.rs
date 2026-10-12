#[path = "../../../support/metadata/maturity/entry.rs"]
mod entry;
#[path = "../../../support/metadata/maturity/field.rs"]
mod field;
#[path = "../../../support/metadata/maturity/required.rs"]
mod required;
#[path = "../../../support/metadata/maturity/value.rs"]
mod value;

pub(super) fn static_maturity_from_plugin_toml(
    manifest: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    entry::static_maturity_from_plugin_toml(manifest)
}
