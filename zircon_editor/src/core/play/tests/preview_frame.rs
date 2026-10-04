use std::sync::Arc;

use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrRuntimeSessionHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::*;

#[test]
fn preview_frame_identity_retains_the_complete_gateway_origin() {
    let gateway = GatewaySessionIdentity::new(
        7,
        ZrRuntimeSessionHandle::new(11),
        13,
        Some(Arc::from("res://project")),
    )
    .with_gateway_generation(17)
    .with_play_instance(Some(19));
    let frame =
        EditorRuntimeFrame::new(ZIRCON_RUNTIME_ABI_VERSION_V1, 1, 1, 23, vec![0, 0, 0, 255]);

    let preview =
        PlayPreviewFrame::copy_and_release(PlayInstanceId::for_test(19), gateway.clone(), frame)
            .expect("preview copy");

    assert_eq!(preview.identity().gateway(), &gateway);
    assert_eq!(preview.identity().instance(), PlayInstanceId::for_test(19));
    assert_eq!(preview.identity().generation(), 23);
    assert_eq!(preview.identity().size(), (1, 1));
    assert_eq!(
        preview.identity().resource_scope("preview-test"),
        "preview-test:19:7:11:13:17:some:19:1:1:23:some:res://project"
    );
}
