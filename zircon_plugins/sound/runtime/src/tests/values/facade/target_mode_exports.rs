// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_facade_reexports_semantic_helper() {
    assert!(
        TARGET_MODE_ROOT.contains("use module::module_target_mode_list_from_plugin_toml"),
        "target_mode parent should expose semantic helpers through child re-exports"
    );
}
