// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_facade_does_not_own_projection_forwarding_body() {
    assert!(
        !MODULE_KIND_ROOT.contains("fn module_kind_value_from_plugin_toml"),
        "module_kind parent must not own semantic projection forwarding bodies"
    );
}
