// 功能字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
mod enabled_by_default;
mod packaging;

use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn parse_feature_defaults_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) -> bool {
    if packaging::parse_default_packaging_line(line, feature) {
        return true;
    }

    enabled_by_default::parse_enabled_by_default_line(line, feature)
}
