use super::*;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchHostRequest, UiDispatchHostRequestKind, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiTreeId},
    text::UiRichLinkTarget,
};

#[test]
fn queue_preserves_generic_reply_order_and_excludes_text_service_requests() {
    let mut queue = RuntimeUiHostRequestQueue::default();
    let mut result = dispatch_result(17);
    result.host_requests.extend([
        host_request(UiDispatchHostRequestKind::ActivateLink {
            target: UiNodeId::new(7),
            link_target: UiRichLinkTarget::parse("res://docs/queue-order.zui").unwrap(),
        }),
        host_request(UiDispatchHostRequestKind::HighPrecisionPointer {
            target: UiNodeId::new(8),
            enabled: true,
        }),
    ]);

    queue.record_result(3, &UiTreeId::new("runtime.ui.host"), &result);
    let mut drained = Vec::new();
    queue.drain_into(&mut drained);

    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].input_sequence, 17);
    assert_eq!(drained[0].request_index, 0);
    assert_eq!(drained[1].request_index, 1);
    assert_eq!(drained[0].target_surface, 3);
}

#[test]
fn queue_is_bounded_by_the_host_page_row_budget() {
    let mut queue = RuntimeUiHostRequestQueue::default();
    let mut result = dispatch_result(1);
    for index in 0..=MAX_PENDING_UI_HOST_REQUESTS {
        result.host_requests.push(host_request(
            UiDispatchHostRequestKind::HighPrecisionPointer {
                target: UiNodeId::new(index as u64 + 1),
                enabled: true,
            },
        ));
    }

    queue.record_result(0, &UiTreeId::new("runtime.ui.host.rows"), &result);
    let mut drained = Vec::new();
    queue.drain_into(&mut drained);

    assert_eq!(drained.len(), MAX_PENDING_UI_HOST_REQUESTS);
}

#[test]
fn queue_rejects_rows_and_aggregate_bytes_beyond_the_encoded_budget() {
    let mut queue = RuntimeUiHostRequestQueue::default();
    let mut oversized = dispatch_result(1);
    oversized
        .host_requests
        .push(host_request(UiDispatchHostRequestKind::ActivateLink {
            target: UiNodeId::new(7),
            link_target: UiRichLinkTarget::parse(&format!(
                "res://oversized/{}",
                "x".repeat(MAX_UI_HOST_REQUEST_ENCODED_BYTES)
            ))
            .unwrap(),
        }));
    queue.record_result(0, &UiTreeId::new("runtime.ui.host.bytes"), &oversized);

    let mut aggregate = dispatch_result(2);
    for index in 0..8 {
        aggregate
            .host_requests
            .push(host_request(UiDispatchHostRequestKind::ActivateLink {
                target: UiNodeId::new(index + 1),
                link_target: UiRichLinkTarget::parse(&format!(
                    "res://aggregate/{index}/{}",
                    "y".repeat(48 * 1024)
                ))
                .unwrap(),
            }));
    }
    queue.record_result(0, &UiTreeId::new("runtime.ui.host.bytes"), &aggregate);
    let mut drained = Vec::new();
    queue.drain_into(&mut drained);

    assert!(!drained.is_empty());
    assert!(drained.len() < 8);
    assert!(
        drained
            .iter()
            .map(|request| serde_json::to_vec(request).unwrap().len())
            .sum::<usize>()
            <= MAX_PENDING_UI_HOST_REQUEST_ENCODED_BYTES
    );
}

fn dispatch_result(sequence: u64) -> UiInputDispatchResult {
    UiInputDispatchResult::new(
        UiInputEvent::Text(UiTextInputEvent {
            metadata: UiInputEventMetadata::new(
                UiInputTimestamp::from_micros(sequence),
                UiInputSequence::new(sequence),
            ),
            text: String::new(),
        }),
        UiDispatchReply::handled(),
    )
}

fn host_request(request: UiDispatchHostRequestKind) -> UiDispatchHostRequest {
    UiDispatchHostRequest {
        effect_index: 7,
        request,
        reason: "typed runtime UI host request test".to_string(),
    }
}
