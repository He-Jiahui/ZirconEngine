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
#[path = "tests/clipboard_host_requests.rs"]
mod tests;
