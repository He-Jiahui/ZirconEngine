// 功能字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super::super) fn parse_optional_feature_line(
    line: &str,
    feature: &mut PendingOptionalFeatureManifest,
) {
    super::dispatch::parse_feature_line(line, feature);
}
