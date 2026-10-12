// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_reexports_semantic_helpers() {
    assert!(
        STRING_ROOT.contains("use dependency::{")
            && STRING_ROOT.contains("use feature::{")
            && STRING_ROOT.contains("use module::{"),
        "string parent should expose semantic field helpers through child re-exports"
    );
}
