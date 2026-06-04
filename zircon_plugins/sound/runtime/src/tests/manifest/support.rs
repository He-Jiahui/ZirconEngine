mod contributions;
mod metadata;
mod options;
mod values;

pub(in crate::tests::manifest) use contributions::StaticSoundContributions;
pub(in crate::tests::manifest) use metadata::StaticSoundPluginMetadata;

pub(super) const STATIC_SOUND_PLUGIN_MANIFEST: &str = include_str!("../../../../plugin.toml");

pub(super) fn static_sound_contributions(manifest: &str) -> StaticSoundContributions {
    contributions::static_sound_contributions(manifest)
}

pub(super) fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    metadata::static_plugin_metadata(manifest)
}

pub(super) fn option_keys_from_plugin_toml(manifest: &str) -> Vec<String> {
    options::option_keys_from_plugin_toml(manifest)
}

pub(super) fn option_manifests_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    options::option_manifests_from_plugin_toml(manifest)
}

pub(super) fn option_manifest_tuple(
    option: zircon_runtime::plugin::PluginOptionManifest,
) -> (String, String, String, String, Vec<String>, Option<String>) {
    options::option_manifest_tuple(option)
}
