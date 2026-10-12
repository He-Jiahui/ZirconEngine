#[path = "../../parser/line/feature/display_name/field.rs"]
mod field;
#[path = "../../parser/line/feature/display_name/state.rs"]
mod state;
#[path = "../../parser/line/feature/display_name/value.rs"]
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
