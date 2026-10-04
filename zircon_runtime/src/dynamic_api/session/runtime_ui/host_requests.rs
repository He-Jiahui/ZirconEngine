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
#[path = "tests/host_requests.rs"]
mod tests;
