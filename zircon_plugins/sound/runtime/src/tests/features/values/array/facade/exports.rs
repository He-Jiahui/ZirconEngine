// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_facade_reexports_semantic_list_helpers() {
    assert!(
        ARRAY_ROOT.contains("use feature::feature_capability_list_from_plugin_toml")
            && ARRAY_ROOT.contains("use list::string_list_from_plugin_toml")
            && ARRAY_ROOT.contains("use module::module_capability_list_from_plugin_toml"),
        "array parent should expose semantic list helpers through child re-exports"
    );
}
