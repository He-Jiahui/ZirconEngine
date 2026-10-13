//! 验证基础会话身份经 Editor 发布投影后保留全部调用方标识。
//! 夹具使用不同数值区分构造字段与两个投影字段；不触发真实会话、gateway 或资源操作。

use std::sync::Arc;

use super::GatewaySessionIdentity;
use crate::ZrRuntimeSessionHandle;

#[test]
/// 用逐字段断言核对基础构造与两次消费式投影的组合结果。
fn gateway_session_identity_preserves_runtime_owner_and_editor_projection() {
    // 各标识取不同值，使投影意外覆盖所属实例、会话或传输 epoch 时可被断言区分。
    let identity = GatewaySessionIdentity::new(
        7,
        ZrRuntimeSessionHandle::new(11),
        13,
        Some(Arc::from("E:/Projects/IdentityFixture")),
    )
    .with_gateway_generation(17)
    .with_play_instance(Some(19));

    assert_eq!(identity.runtime_instance(), 7);
    assert_eq!(identity.runtime_session(), ZrRuntimeSessionHandle::new(11));
    assert_eq!(identity.transport_epoch(), 13);
    assert_eq!(identity.project(), Some("E:/Projects/IdentityFixture"));
    assert_eq!(identity.gateway_generation(), 17);
    assert_eq!(identity.play_instance(), Some(19));
}
