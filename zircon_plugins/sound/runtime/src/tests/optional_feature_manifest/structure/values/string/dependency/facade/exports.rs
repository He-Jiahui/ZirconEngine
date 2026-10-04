// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_facade_reexports_field_helpers() {
    assert!(
        STRING_DEPENDENCY.contains("use capability::dependency_capability_string_from_plugin_toml")
            && STRING_DEPENDENCY
                .contains("use plugin_id::dependency_plugin_id_string_from_plugin_toml"),
        "dependency string domain should expose child-owned field helpers"
    );
}
