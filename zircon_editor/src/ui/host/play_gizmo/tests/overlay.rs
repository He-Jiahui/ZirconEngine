use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

use crate::core::play::{PlayInstanceId, PlayPreviewFrame};

#[test]
fn overlay_resource_scope_pins_complete_frame_world_and_entity_identity() {
    let frame = PlayPreviewFrame::for_test(
        PlayInstanceId::for_test(7),
        GatewaySessionIdentity::new(11, ZrRuntimeSessionHandle::new(13), 17, None)
            .with_gateway_generation(19)
            .with_play_instance(Some(7)),
        640,
        360,
        23,
        vec![0; 640 * 360 * 4],
    );

    assert_eq!(
        frame.identity().resource_scope("play-gizmo:29:31"),
        "play-gizmo:29:31:7:11:13:17:19:some:7:640:360:23:none"
    );
}
