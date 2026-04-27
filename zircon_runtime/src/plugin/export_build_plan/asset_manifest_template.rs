use crate::asset::project::ProjectManifest;

pub(super) fn asset_manifest_template(manifest: &ProjectManifest) -> String {
    toml::to_string_pretty(manifest)
        .unwrap_or_else(|error| format!("# failed to serialize zircon project manifest: {error}\n"))
}
