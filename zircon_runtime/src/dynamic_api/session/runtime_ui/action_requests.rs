use std::collections::VecDeque;

use zircon_runtime_interface::ui::component::{UiComponentEvent, UiSecureTextValueRef, UiValue};
use zircon_runtime_interface::ui::dispatch::{UiInputDispatchResult, UiInputEvent};
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::{
    ZrRuntimeUiActionHostRequestV1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1, ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1,
};

const MAX_PENDING_UI_ACTION_REQUESTS: usize = ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_items;
const MAX_UI_ACTION_REQUEST_ENCODED_BYTES: usize = 64 * 1024;
const UI_ACTION_OUTPUT_ENVELOPE_RESERVE_BYTES: usize = 16 * 1024;
const MAX_PENDING_UI_ACTION_ENCODED_BYTES: usize = ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1
    .max_encoded_bytes
    - UI_ACTION_OUTPUT_ENVELOPE_RESERVE_BYTES;
const UI_ACTION_OUTPUT_ENVELOPE_RESERVED_NESTING: usize = 32;
const MAX_UI_ACTION_PAYLOAD_NESTING: usize =
    (ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1 - UI_ACTION_OUTPUT_ENVELOPE_RESERVED_NESTING) / 2;

#[derive(Default)]
pub(super) struct RuntimeUiActionRequestQueue {
    pending: VecDeque<QueuedUiActionRequest>,
    pending_encoded_bytes: usize,
}

struct QueuedUiActionRequest {
    request: ZrRuntimeUiActionHostRequestV1,
    encoded_len: usize,
}

impl RuntimeUiActionRequestQueue {
    pub(super) fn record_result(
        &mut self,
        target_surface: u32,
        tree_id: &UiTreeId,
        result: &UiInputDispatchResult,
    ) -> Vec<UiSecureTextValueRef> {
        let input_sequence = input_sequence(&result.event);
        let mut admitted = 0_usize;
        let mut rejected_full = 0_usize;
        let mut rejected_oversized = 0_usize;
        let mut rejected_secure_payload = 0_usize;
        let mut superseded_secure_change = 0_usize;
        let mut revoked_secure_values = Vec::new();

        for (action_index, report) in result.component_events.iter().enumerate() {
            let remaining_events = result.component_events.len().saturating_sub(action_index);
            if !report.delivered {
                if let Some(reference) = secure_value(&report.event) {
                    append_revoked_secure_value(
                        &mut revoked_secure_values,
                        Some(reference.clone()),
                        remaining_events,
                    );
                }
                continue;
            }
            let Some(invocation) = report.template_action.as_ref() else {
                continue;
            };
            let secure_value = secure_value(&report.event).cloned();
            if secure_value.as_ref().is_some_and(|reference| {
                reference.tree_id() != tree_id || reference.node_id() != report.target
            }) {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    secure_value,
                    remaining_events,
                );
                rejected_secure_payload = rejected_secure_payload.saturating_add(1);
                continue;
            }
            // SecureValue 只允许携带不含明文的空值载荷；拒收路径把引用交回调用者撤销。
            if secure_value.is_some()
                && invocation
                    .payload
                    .values()
                    .any(|value| !matches!(value, UiValue::Null))
            {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    secure_value,
                    remaining_events,
                );
                rejected_secure_payload = rejected_secure_payload.saturating_add(1);
                continue;
            }
            if invocation
                .payload
                .values()
                .any(|value| !value_nesting_within(value, 0))
            {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    secure_value,
                    remaining_events,
                );
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            if matches!(&report.event, UiComponentEvent::SecureValueChanged { .. }) {
                if let Some(reference) = secure_value.as_ref() {
                    superseded_secure_change = superseded_secure_change.saturating_add(
                        self.supersede_secure_action(target_surface, invocation, reference),
                    );
                }
            }
            if self.pending.len() >= MAX_PENDING_UI_ACTION_REQUESTS {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    secure_value,
                    remaining_events,
                );
                rejected_full = rejected_full.saturating_add(1);
                continue;
            }
            let request = ZrRuntimeUiActionHostRequestV1::new(
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                target_surface,
                input_sequence,
                u32::try_from(action_index).unwrap_or(u32::MAX),
                tree_id.clone(),
                report.target,
                invocation.clone(),
                secure_value,
            );
            let encoded_len = match serde_json::to_vec(&request) {
                Ok(encoded) => encoded.len(),
                Err(_) => {
                    append_revoked_secure_value(
                        &mut revoked_secure_values,
                        request.secure_value.clone(),
                        remaining_events,
                    );
                    rejected_oversized = rejected_oversized.saturating_add(1);
                    continue;
                }
            };
            if encoded_len > MAX_UI_ACTION_REQUEST_ENCODED_BYTES {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    request.secure_value.clone(),
                    remaining_events,
                );
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            let Some(next_encoded_bytes) = self.pending_encoded_bytes.checked_add(encoded_len)
            else {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    request.secure_value.clone(),
                    remaining_events,
                );
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            };
            if next_encoded_bytes > MAX_PENDING_UI_ACTION_ENCODED_BYTES {
                append_revoked_secure_value(
                    &mut revoked_secure_values,
                    request.secure_value.clone(),
                    remaining_events,
                );
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            self.pending.push_back(QueuedUiActionRequest {
                request,
                encoded_len,
            });
            self.pending_encoded_bytes = next_encoded_bytes;
            admitted = admitted.saturating_add(1);
        }

        crate::profile_counter!("runtime", "ui.action_queue.admitted", admitted);
        crate::profile_counter!("runtime", "ui.action_queue.pending", self.pending.len());
        crate::profile_counter!(
            "runtime",
            "ui.action_queue.pending_encoded_bytes",
            self.pending_encoded_bytes
        );
        crate::profile_counter!("runtime", "ui.action_queue.rejected_full", rejected_full);
        crate::profile_counter!(
            "runtime",
            "ui.action_queue.rejected_oversized",
            rejected_oversized
        );
        crate::profile_counter!(
            "runtime",
            "ui.action_queue.rejected_secure_payload",
            rejected_secure_payload
        );
        crate::profile_counter!(
            "runtime",
            "ui.action_queue.superseded_secure_change",
            superseded_secure_change
        );
        revoked_secure_values
    }

    pub(super) fn drain_into(&mut self, output: &mut Vec<ZrRuntimeUiActionHostRequestV1>) {
        output.reserve(self.pending.len());
        output.extend(self.pending.drain(..).map(|queued| queued.request));
        self.pending_encoded_bytes = 0;
    }

    fn supersede_secure_action(
        &mut self,
        target_surface: u32,
        invocation: &zircon_runtime_interface::ui::dispatch::UiTemplateActionInvocation,
        reference: &UiSecureTextValueRef,
    ) -> usize {
        let previous_len = self.pending.len();
        self.pending.retain(|queued| {
            let request = &queued.request;
            request.target_surface != target_surface
                || request.invocation.is_action() != invocation.is_action()
                || request.invocation.target_id() != invocation.target_id()
                || !request.secure_value.as_ref().is_some_and(|pending| {
                    pending.node_id() == reference.node_id()
                        && pending.property() == reference.property()
                })
        });
        self.pending_encoded_bytes = self
            .pending
            .iter()
            .map(|queued| queued.encoded_len)
            .fold(0_usize, usize::saturating_add);
        previous_len.saturating_sub(self.pending.len())
    }
}

#[inline]
fn append_revoked_secure_value(
    revoked_secure_values: &mut Vec<UiSecureTextValueRef>,
    value: Option<UiSecureTextValueRef>,
    remaining_events: usize,
) {
    let Some(value) = value else {
        return;
    };
    if revoked_secure_values.is_empty() {
        revoked_secure_values.reserve(remaining_events);
    }
    revoked_secure_values.push(value);
}

fn secure_value(event: &UiComponentEvent) -> Option<&UiSecureTextValueRef> {
    match event {
        UiComponentEvent::SecureValueChanged { reference, .. }
        | UiComponentEvent::SecureCommit { reference, .. } => Some(reference),
        _ => None,
    }
}

fn value_nesting_within(value: &UiValue, depth: usize) -> bool {
    if depth > MAX_UI_ACTION_PAYLOAD_NESTING {
        return false;
    }
    match value {
        UiValue::Array(values) => values
            .iter()
            .all(|value| value_nesting_within(value, depth.saturating_add(1))),
        UiValue::Map(values) => values
            .values()
            .all(|value| value_nesting_within(value, depth.saturating_add(1))),
        _ => true,
    }
}

pub(super) fn input_sequence(event: &UiInputEvent) -> u64 {
    match event {
        UiInputEvent::Pointer(event) => event.metadata.sequence.0,
        UiInputEvent::Keyboard(event) => event.metadata.sequence.0,
        UiInputEvent::Text(event) => event.metadata.sequence.0,
        UiInputEvent::Ime(event) => event.metadata.sequence.0,
        UiInputEvent::Clipboard(event) => event.metadata.sequence.0,
        UiInputEvent::Navigation(event) => event.metadata.sequence.0,
        UiInputEvent::Analog(event) => event.metadata.sequence.0,
        UiInputEvent::MouseMotion(event) => event.metadata.sequence.0,
        UiInputEvent::DragDrop(event) => event.metadata.sequence.0,
        UiInputEvent::Popup(event) => event.metadata.sequence.0,
        UiInputEvent::TooltipTimer(event) => event.metadata.sequence.0,
        UiInputEvent::TypeaheadTimer(event) => event.metadata.sequence.0,
        UiInputEvent::SubmenuHoverTimer(event) => event.metadata.sequence.0,
        UiInputEvent::ToastTimer(event) => event.metadata.sequence.0,
        UiInputEvent::Accessibility(event) => event.metadata.sequence.0,
    }
}

#[cfg(test)]
#[path = "tests/action_requests.rs"]
mod tests;
