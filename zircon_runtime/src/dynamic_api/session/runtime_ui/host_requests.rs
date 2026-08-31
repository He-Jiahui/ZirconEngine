use std::collections::VecDeque;

use zircon_runtime_interface::ui::dispatch::UiInputDispatchResult;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::{
    ZrRuntimeUiHostRequestV1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
};

use super::action_requests::input_sequence;

const MAX_PENDING_UI_HOST_REQUESTS: usize = ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_items;
const MAX_UI_HOST_REQUEST_ENCODED_BYTES: usize = 64 * 1024;
const UI_HOST_OUTPUT_ENVELOPE_RESERVE_BYTES: usize = 16 * 1024;
const MAX_PENDING_UI_HOST_REQUEST_ENCODED_BYTES: usize = ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1
    .max_encoded_bytes
    - UI_HOST_OUTPUT_ENVELOPE_RESERVE_BYTES;

#[derive(Default)]
pub(super) struct RuntimeUiHostRequestQueue {
    pending: VecDeque<QueuedUiHostRequest>,
    pending_encoded_bytes: usize,
}

struct QueuedUiHostRequest {
    request: ZrRuntimeUiHostRequestV1,
}

impl RuntimeUiHostRequestQueue {
    pub(super) fn record_result(
        &mut self,
        target_surface: u32,
        tree_id: &UiTreeId,
        result: &UiInputDispatchResult,
    ) {
        let input_sequence = input_sequence(&result.event);
        let mut admitted = 0_usize;
        let mut rejected_full = 0_usize;
        let mut rejected_oversized = 0_usize;

        for (request_index, dispatch_request) in result.host_requests.iter().enumerate() {
            let Some(request) = ZrRuntimeUiHostRequestV1::from_dispatch_request(
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                target_surface,
                input_sequence,
                u32::try_from(request_index).unwrap_or(u32::MAX),
                tree_id.clone(),
                u32::try_from(dispatch_request.effect_index).unwrap_or(u32::MAX),
                &dispatch_request.request,
            ) else {
                continue;
            };
            if self.pending.len() >= MAX_PENDING_UI_HOST_REQUESTS {
                rejected_full = rejected_full.saturating_add(1);
                continue;
            }
            let encoded_len = match serde_json::to_vec(&request) {
                Ok(encoded) => encoded.len(),
                Err(_) => {
                    rejected_oversized = rejected_oversized.saturating_add(1);
                    continue;
                }
            };
            if encoded_len > MAX_UI_HOST_REQUEST_ENCODED_BYTES {
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            let Some(next_encoded_bytes) = self.pending_encoded_bytes.checked_add(encoded_len)
            else {
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            };
            if next_encoded_bytes > MAX_PENDING_UI_HOST_REQUEST_ENCODED_BYTES {
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            self.pending.push_back(QueuedUiHostRequest { request });
            self.pending_encoded_bytes = next_encoded_bytes;
            admitted = admitted.saturating_add(1);
        }

        crate::profile_counter!("runtime", "ui.host_queue.admitted", admitted);
        crate::profile_counter!("runtime", "ui.host_queue.pending", self.pending.len());
        crate::profile_counter!(
            "runtime",
            "ui.host_queue.pending_encoded_bytes",
            self.pending_encoded_bytes
        );
        crate::profile_counter!("runtime", "ui.host_queue.rejected_full", rejected_full);
        crate::profile_counter!(
            "runtime",
            "ui.host_queue.rejected_oversized",
            rejected_oversized
        );
    }

    pub(super) fn drain_into(&mut self, output: &mut Vec<ZrRuntimeUiHostRequestV1>) {
        output.reserve(self.pending.len());
        output.extend(self.pending.drain(..).map(|queued| queued.request));
        self.pending_encoded_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiDispatchHostRequest, UiDispatchHostRequestKind, UiDispatchReply,
            UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputSequence,
            UiInputTimestamp, UiTextInputEvent,
        },
        event_ui::{UiNodeId, UiTreeId},
    };

    #[test]
    fn queue_preserves_generic_reply_order_and_excludes_text_service_requests() {
        let mut queue = RuntimeUiHostRequestQueue::default();
        let mut result = dispatch_result(17);
        result.host_requests.extend([
            host_request(UiDispatchHostRequestKind::ActivateLink {
                target: UiNodeId::new(7),
                href: "https://example.invalid/docs".to_string(),
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
                href: "x".repeat(MAX_UI_HOST_REQUEST_ENCODED_BYTES),
            }));
        queue.record_result(0, &UiTreeId::new("runtime.ui.host.bytes"), &oversized);

        let mut aggregate = dispatch_result(2);
        for index in 0..8 {
            aggregate
                .host_requests
                .push(host_request(UiDispatchHostRequestKind::ActivateLink {
                    target: UiNodeId::new(index + 1),
                    href: "y".repeat(48 * 1024),
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
}
