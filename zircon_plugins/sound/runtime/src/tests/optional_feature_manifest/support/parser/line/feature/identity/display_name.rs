mod field;
mod state;
mod value;

use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_display_name_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::feature_display_name_value(line) else {
        return false;
    };
    state::set_feature_display_name(feature, value::feature_display_name_from_plugin_toml(value));
    true
}
