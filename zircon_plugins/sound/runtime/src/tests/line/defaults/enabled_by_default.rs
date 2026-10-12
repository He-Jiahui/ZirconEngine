#[path = "../../parser/line/enabled_by_default/field.rs"]
mod field;
#[path = "../../parser/line/enabled_by_default/state.rs"]
mod state;
#[path = "../../parser/line/enabled_by_default/value.rs"]
mod value;

use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_enabled_by_default_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::enabled_by_default_value(line) else {
        return false;
    };
    state::set_enabled_by_default(feature, value::enabled_by_default_from_plugin_toml(value));
    true
}
