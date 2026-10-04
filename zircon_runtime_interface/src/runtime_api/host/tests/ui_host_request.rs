use super::*;
use crate::ui::{
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiClipboardTransferId,
        UiClipboardTransferIntent, UiDispatchHostRequestKind, UiPointerLockPolicy,
    },
    event_ui::{UiNodeId, UiTreeId},
};
use crate::{ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

#[test]
fn generic_ui_host_request_maps_platform_reply_identity() {
    let request = ZrRuntimeUiHostRequestV1::from_dispatch_request(
        ZrRuntimeViewportHandle::new(3),
        5,
        11,
        2,
        UiTreeId::new("runtime.ui.host"),
        7,
        &UiDispatchHostRequestKind::PointerLock {
            target: UiNodeId::new(13),
            policy: UiPointerLockPolicy::RawDelta,
        },
    )
    .expect("pointer lock is a generic host request");

    assert_eq!(request.target_viewport, ZrRuntimeViewportHandle::new(3));
    assert_eq!(request.target_surface, 5);
    assert_eq!(request.input_sequence, 11);
    assert_eq!(request.request_index, 2);
    assert_eq!(request.effect_index, 7);
    assert!(matches!(
        request.kind,
        ZrRuntimeUiHostRequestKindV1::PointerLock {
            target,
            policy: UiPointerLockPolicy::RawDelta,
        } if target == UiNodeId::new(13)
    ));
}

#[test]
fn text_service_requests_remain_with_their_dedicated_queues() {
    let clipboard = UiDispatchHostRequestKind::Clipboard(UiClipboardRequest {
        transfer_id: UiClipboardTransferId::issue(),
        intent: UiClipboardTransferIntent::Paste,
        expected_edit_revision: 1,
        kind: UiClipboardRequestKind::ReadText,
        owner: UiNodeId::new(7),
        text: None,
    });

    assert!(ZrRuntimeUiHostRequestKindV1::from_dispatch_request(&clipboard).is_none());
}

#[test]
fn generic_ui_host_request_round_trips_without_debugging_dynamic_content() {
    let dynamic_link_target = UiRichLinkTarget::parse("res://private/token-never-log.zui").unwrap();
    let request = ZrRuntimeUiHostRequestV1::from_dispatch_request(
        ZrRuntimeViewportHandle::new(3),
        5,
        11,
        2,
        UiTreeId::new("runtime.ui.host"),
        7,
        &UiDispatchHostRequestKind::ActivateLink {
            target: UiNodeId::new(13),
            link_target: dynamic_link_target.clone(),
        },
    )
    .expect("approved link activation is a generic host request");
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![ZrRuntimeHostRequestV1::ui_host(request.clone())],
    );

    let encoded = serde_json::to_vec(&batch).expect("serialize generic UI host request");
    let encoded_text = std::str::from_utf8(&encoded).expect("host request JSON is UTF-8");
    let decoded: ZrRuntimeHostRequestBatchV1 =
        serde_json::from_slice(&encoded).expect("deserialize generic UI host request");

    assert_eq!(decoded, batch);
    assert!(encoded_text.contains("\"href\""));
    assert!(!encoded_text.contains("\"link_target\""));
    assert!(matches!(
        &request.kind,
        ZrRuntimeUiHostRequestKindV1::ActivateLink {
            link_target,
            ..
        } if link_target == &dynamic_link_target
    ));
    assert!(!format!("{request:?}").contains(&dynamic_link_target.to_string()));
}
