use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super::super) fn parse_optional_feature_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) {
    super::dispatch::parse_feature_line(line, feature);
}
