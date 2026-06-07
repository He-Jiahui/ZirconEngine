mod capability_statuses;
mod entry;
mod maturity;
mod types;

pub(in crate::tests::manifest) use types::StaticSoundPluginMetadata;

pub(super) fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    entry::static_plugin_metadata(manifest)
}
