// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_facade_reexports_semantic_helper() {
    assert!(
        PACKAGING_ROOT.contains("use defaults::default_packaging_strategy_list_from_plugin_toml"),
        "packaging parent should expose semantic helpers through child re-exports"
    );
}
