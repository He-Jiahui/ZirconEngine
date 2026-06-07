mod field;
mod state;
mod value;

use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_id_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::feature_id_value(line) else {
        return false;
    };
    state::set_feature_id(feature, value::feature_id_from_plugin_toml(value));
    true
}
