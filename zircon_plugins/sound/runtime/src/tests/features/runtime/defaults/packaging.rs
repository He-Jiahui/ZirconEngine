// 源码结构守卫：固定可选功能运行时投影的声明归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_defaults_packaging_child_owns_projection() {
    assert!(
        RUNTIME_DEFAULTS_PACKAGING.contains("feature.default_packaging.clone()"),
        "runtime defaults packaging child should own default-packaging projection"
    );
}
