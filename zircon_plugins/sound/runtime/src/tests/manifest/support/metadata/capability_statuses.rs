mod entry;
mod line;
mod state;

pub(super) fn capability_statuses_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
    entry::capability_statuses_from_plugin_toml(manifest)
}
