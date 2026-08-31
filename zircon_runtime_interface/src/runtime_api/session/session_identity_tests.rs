use std::sync::Arc;

use super::GatewaySessionIdentity;
use crate::ZrRuntimeSessionHandle;

#[test]
fn gateway_session_identity_preserves_runtime_owner_and_editor_projection() {
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
