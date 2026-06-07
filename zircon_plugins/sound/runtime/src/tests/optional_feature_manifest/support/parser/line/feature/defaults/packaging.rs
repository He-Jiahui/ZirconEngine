mod field;
mod state;
mod strategies;

use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_default_packaging_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::default_packaging_value(line) else {
        return false;
    };
    state::set_default_packaging(
        feature,
        strategies::default_packaging_from_plugin_toml(value),
    );
    true
}
