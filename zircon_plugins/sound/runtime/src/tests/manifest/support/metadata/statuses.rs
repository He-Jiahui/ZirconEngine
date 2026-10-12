#[path = "../../../support/metadata/statuses/entry.rs"]
mod entry;
#[path = "../../../support/metadata/statuses/line.rs"]
mod line;
#[path = "../../../support/metadata/statuses/state.rs"]
mod state;

pub(super) fn capability_statuses_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
    entry::capability_statuses_from_plugin_toml(manifest)
}
