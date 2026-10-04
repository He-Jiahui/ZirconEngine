// 功能字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
mod field;
mod state;
mod values;

use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_capability_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    let Some(value) = field::feature_capabilities_value(line) else {
        return false;
    };
    state::set_feature_capabilities(
        feature,
        values::feature_capabilities_from_plugin_toml(value),
    );
    true
}
