use std::collections::VecDeque;

use zircon_runtime_interface::ui::dispatch::{
    UiClipboardRequest, UiClipboardTransferReceipt, UiClipboardTransferStatus,
    UiDispatchHostRequestKind, UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

const MAX_PENDING_CLIPBOARD_HOST_REQUESTS: usize =
    zircon_runtime_interface::ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_items;

#[derive(Default)]
pub(super) struct UiClipboardHostRequestQueue {
    pending: VecDeque<UiClipboardRequest>,
}

impl UiClipboardHostRequestQueue {
    pub(super) fn record_result(
        &mut self,
        surface: &mut UiSurface,
        result: &mut UiInputDispatchResult,
    ) {
        let mut retained = Vec::with_capacity(result.host_requests.len());
        for host_request in std::mem::take(&mut result.host_requests) {
            let UiDispatchHostRequestKind::Clipboard(request) = &host_request.request else {
                retained.push(host_request);
                continue;
            };
            self.pending.retain(|queued| queued.owner != request.owner);
            if self.pending.len() >= MAX_PENDING_CLIPBOARD_HOST_REQUESTS {
                surface.cancel_clipboard_transfer(request.transfer_id);
                result.diagnostics.clipboard_transfer = Some(UiClipboardTransferReceipt {
                    transfer_id: request.transfer_id,
                    intent: Some(request.intent),
                    status: UiClipboardTransferStatus::Failed,
                });
                result
                    .diagnostics
                    .notes
                    .push("clipboard host request queue is full".to_string());
                continue;
            }
            self.pending.push_back(request.clone());
            retained.push(host_request);
        }
        result.host_requests = retained;
    }

    pub(super) fn drain_into(&mut self, output: &mut Vec<UiClipboardRequest>) {
        output.reserve(self.pending.len());
        output.extend(self.pending.drain(..));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiClipboardInputEvent, UiClipboardRequestKind, UiClipboardTransferFailure,
            UiClipboardTransferId, UiClipboardTransferIntent, UiClipboardTransferOutcome,
            UiDispatchHostRequest, UiDispatchReply, UiInputEvent, UiInputEventMetadata,
            UiInputSequence, UiInputTimestamp,
        },
        event_ui::{UiNodeId, UiTreeId},
    };

    #[test]
    fn queue_capacity_is_bounded_by_the_host_output_row_budget() {
        assert_eq!(
            MAX_PENDING_CLIPBOARD_HOST_REQUESTS,
            zircon_runtime_interface::ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_items
        );
    }

    #[test]
    fn same_owner_supersedes_queued_request_before_host_drain() {
        let mut queue = UiClipboardHostRequestQueue::default();
        let mut surface = UiSurface::new(UiTreeId::new("clipboard-queue"));
        let first = clipboard_request(UiNodeId::new(7));
        let second = clipboard_request(UiNodeId::new(7));

        queue.record_result(&mut surface, &mut dispatch_result(first));
        queue.record_result(&mut surface, &mut dispatch_result(second.clone()));
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert_eq!(drained, [second]);
    }

    #[test]
    fn queue_rejects_rows_beyond_its_fixed_capacity() {
        let mut queue = UiClipboardHostRequestQueue::default();
        let mut surface = UiSurface::new(UiTreeId::new("clipboard-queue"));
        for index in 0..MAX_PENDING_CLIPBOARD_HOST_REQUESTS {
            queue.record_result(
                &mut surface,
                &mut dispatch_result(clipboard_request(UiNodeId::new(index as u64 + 1))),
            );
        }
        let overflow = clipboard_request(UiNodeId::new(10_000));
        let overflow_id = overflow.transfer_id;
        let overflow_intent = overflow.intent;
        let mut result = dispatch_result(overflow);

        queue.record_result(&mut surface, &mut result);
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert_eq!(drained.len(), MAX_PENDING_CLIPBOARD_HOST_REQUESTS);
        assert!(result.host_requests.is_empty());
        assert!(
            result
                .diagnostics
                .clipboard_transfer
                .is_some_and(|receipt| {
                    receipt.transfer_id == overflow_id
                        && receipt.intent == Some(overflow_intent)
                        && receipt.status == UiClipboardTransferStatus::Failed
                })
        );
        assert_eq!(
            result.diagnostics.notes,
            ["clipboard host request queue is full"]
        );
    }

    fn clipboard_request(owner: UiNodeId) -> UiClipboardRequest {
        UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent: UiClipboardTransferIntent::Paste,
            expected_edit_revision: 1,
            kind: UiClipboardRequestKind::ReadText,
            owner,
            text: None,
        }
    }

    fn dispatch_result(request: UiClipboardRequest) -> UiInputDispatchResult {
        let mut result = UiInputDispatchResult::new(
            UiInputEvent::Clipboard(UiClipboardInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(1),
                    UiInputSequence::new(1),
                ),
                transfer_id: request.transfer_id,
                owner: request.owner,
                outcome: UiClipboardTransferOutcome::Failed {
                    reason: UiClipboardTransferFailure::Cancelled,
                },
            }),
            UiDispatchReply::default(),
        );
        result.host_requests.push(UiDispatchHostRequest {
            effect_index: 0,
            request: UiDispatchHostRequestKind::Clipboard(request),
            reason: "clipboard test".to_string(),
        });
        result
    }
}
