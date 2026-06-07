mod capabilities;
mod defaults;
mod dispatch;
mod identity;

use super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super) fn parse_optional_feature_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) {
    dispatch::parse_feature_line(line, feature);
}
