use super::super::types::StaticOptionalFeatureManifest;
use super::state::OptionalFeatureParserState;

pub(in super::super) fn optional_features_from_plugin_toml(
    manifest: &str,
) -> Vec<StaticOptionalFeatureManifest> {
    let mut parser = OptionalFeatureParserState::default();
    for line in manifest.lines().map(str::trim) {
        parser.parse_manifest_line(line);
    }
    parser.finish()
}
