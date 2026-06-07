mod enabled_by_default;
mod packaging;

use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_defaults_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    if packaging::parse_default_packaging_line(line, feature) {
        return true;
    }

    enabled_by_default::parse_enabled_by_default_line(line, feature)
}
