mod line;
mod pending;
mod section;
mod state;

use self::state::OptionalFeatureParserState;
use super::types::StaticOptionalFeatureManifest;

pub(super) fn optional_features_from_plugin_toml(
    manifest: &str,
) -> Vec<StaticOptionalFeatureManifest> {
    let mut parser = OptionalFeatureParserState::default();
    for line in manifest.lines().map(str::trim) {
        parser.parse_manifest_line(line);
    }
    parser.finish()
}
