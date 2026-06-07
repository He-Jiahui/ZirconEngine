use super::super::state;

pub(in crate::tests::manifest::support) fn option_manifests_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    let mut state = state::OptionManifestParserState::default();
    for line in manifest.lines().map(str::trim) {
        state.parse_manifest_line(line);
    }
    state.finish()
}
