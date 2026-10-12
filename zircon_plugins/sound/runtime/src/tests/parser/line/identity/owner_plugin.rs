#[path = "../feature/owner_plugin/field.rs"]
mod field;
#[path = "../feature/owner_plugin/state.rs"]
mod state;
#[path = "../feature/owner_plugin/value.rs"]
mod value;

use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_owner_plugin_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::feature_owner_plugin_value(line) else {
        return false;
    };
    state::set_feature_owner_plugin_id(
        feature,
        value::feature_owner_plugin_from_plugin_toml(value),
    );
    true
}
