mod display_name;
mod id;
mod owner_plugin;

use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_identity_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    if id::parse_feature_id_line(line, feature) {
        return true;
    }

    if display_name::parse_feature_display_name_line(line, feature) {
        return true;
    }

    owner_plugin::parse_feature_owner_plugin_line(line, feature)
}
