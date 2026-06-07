use super::super::super::super::types::PendingOptionalFeatureManifest;
use super::{capabilities, defaults, identity};

pub(super) fn parse_feature_line(line: &str, feature: &mut PendingOptionalFeatureManifest) {
    if identity::parse_feature_identity_line(line, feature) {
        return;
    }

    if capabilities::parse_feature_capability_line(line, feature) {
        return;
    }

    if defaults::parse_feature_defaults_line(line, feature) {
        return;
    }
}
