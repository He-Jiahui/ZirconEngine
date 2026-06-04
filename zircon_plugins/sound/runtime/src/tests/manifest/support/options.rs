mod keys;
mod parser;
mod projection;
mod state;

pub(super) fn option_keys_from_plugin_toml(manifest: &str) -> Vec<String> {
    keys::option_keys_from_plugin_toml(manifest)
}

pub(super) fn option_manifests_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    parser::option_manifests_from_plugin_toml(manifest)
}

pub(super) fn option_manifest_tuple(
    option: zircon_runtime::plugin::PluginOptionManifest,
) -> (String, String, String, String, Vec<String>, Option<String>) {
    projection::option_manifest_tuple(option)
}
