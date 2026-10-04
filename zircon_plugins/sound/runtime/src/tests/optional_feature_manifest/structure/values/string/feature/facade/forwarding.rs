// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_facade_does_not_own_forwarding_bodies() {
    assert!(
        !STRING_FEATURE.contains("fn feature_id_string_from_plugin_toml")
            && !STRING_FEATURE.contains("fn feature_display_name_string_from_plugin_toml")
            && !STRING_FEATURE.contains("fn feature_owner_plugin_string_from_plugin_toml"),
        "feature string domain must not own semantic field forwarding bodies"
    );
}
