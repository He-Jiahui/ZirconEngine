// 源码结构守卫：固定可选功能运行时投影的声明归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_identity_id_child_owns_projection() {
    assert!(
        RUNTIME_IDENTITY_ID.contains("feature.id.clone()"),
        "runtime identity id child should own id projection"
    );
}
