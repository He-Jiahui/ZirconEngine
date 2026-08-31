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
            if !report.delivered {
                if let Some(reference) = secure_value(&report.event) {
                    revoked_secure_values.push(reference.clone());
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
                revoked_secure_values.extend(secure_value);
                rejected_secure_payload = rejected_secure_payload.saturating_add(1);
                continue;
            }
            if secure_value.is_some()
                && invocation
                    .payload
                    .values()
                    .any(|value| !matches!(value, UiValue::Null))
            {
                revoked_secure_values.extend(secure_value);
                rejected_secure_payload = rejected_secure_payload.saturating_add(1);
                continue;
            }
            if invocation
                .payload
                .values()
                .any(|value| !value_nesting_within(value, 0))
            {
                revoked_secure_values.extend(secure_value);
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
                revoked_secure_values.extend(secure_value);
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
                    revoked_secure_values.extend(request.secure_value.clone());
                    rejected_oversized = rejected_oversized.saturating_add(1);
                    continue;
                }
            };
            if encoded_len > MAX_UI_ACTION_REQUEST_ENCODED_BYTES {
                revoked_secure_values.extend(request.secure_value.clone());
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            }
            let Some(next_encoded_bytes) = self.pending_encoded_bytes.checked_add(encoded_len)
            else {
                revoked_secure_values.extend(request.secure_value.clone());
                rejected_oversized = rejected_oversized.saturating_add(1);
                continue;
            };
            if next_encoded_bytes > MAX_PENDING_UI_ACTION_ENCODED_BYTES {
                revoked_secure_values.extend(request.secure_value.clone());
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
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use zircon_runtime_interface::ui::component::UiSecureTextValueRef;
    use zircon_runtime_interface::ui::dispatch::{
        UiComponentEventReport, UiDispatchReply, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiTemplateActionInvocation, UiTextInputEvent,
    };
    use zircon_runtime_interface::ui::event_ui::UiNodeId;

    #[test]
    fn action_queue_is_bounded_by_the_host_page_row_budget() {
        let tree_id = UiTreeId::new("queue.bound");
        let mut queue = RuntimeUiActionRequestQueue::default();
        for sequence in 0..=MAX_PENDING_UI_ACTION_REQUESTS {
            let _ = queue.record_result(0, &tree_id, &action_result(sequence as u64));
        }
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert_eq!(drained.len(), MAX_PENDING_UI_ACTION_REQUESTS);
    }

    #[test]
    fn action_queue_is_bounded_by_the_host_page_encoded_byte_budget() {
        let tree_id = UiTreeId::new("queue.byte-bound");
        let mut queue = RuntimeUiActionRequestQueue::default();
        for sequence in 0..10 {
            let mut result = action_result(sequence);
            result.component_events[0]
                .template_action
                .as_mut()
                .unwrap()
                .payload
                .insert("blob".to_string(), UiValue::String("x".repeat(48 * 1024)));
            let _ = queue.record_result(0, &tree_id, &result);
        }

        assert!(queue.pending.len() < 10);
        assert!(queue.pending_encoded_bytes <= MAX_PENDING_UI_ACTION_ENCODED_BYTES);
        assert_eq!(
            queue.pending_encoded_bytes,
            queue
                .pending
                .iter()
                .map(|queued| queued.encoded_len)
                .sum::<usize>()
        );
    }

    #[test]
    fn action_queue_rejects_payloads_beyond_the_json_nesting_budget() {
        let tree_id = UiTreeId::new("queue.depth-bound");
        let mut result = action_result(1);
        let mut nested = UiValue::Null;
        for _ in 0..=MAX_UI_ACTION_PAYLOAD_NESTING {
            nested = UiValue::Array(vec![nested]);
        }
        result.component_events[0]
            .template_action
            .as_mut()
            .unwrap()
            .payload
            .insert("nested".to_string(), nested);
        let mut queue = RuntimeUiActionRequestQueue::default();

        assert!(queue.record_result(0, &tree_id, &result).is_empty());
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert!(drained.is_empty());
    }

    #[test]
    fn secure_action_with_non_redacted_payload_fails_closed() {
        let tree_id = UiTreeId::new("queue.secure");
        let target = UiNodeId::new(7);
        let mut result = action_result(1);
        result.component_events[0].event = UiComponentEvent::SecureCommit {
            property: "value".to_string(),
            reference: UiSecureTextValueRef::issue(tree_id.clone(), target, "value"),
        };
        result.component_events[0]
            .template_action
            .as_mut()
            .unwrap()
            .payload
            .insert(
                "credential".to_string(),
                UiValue::String("must-not-cross-boundary".to_string()),
            );
        let mut queue = RuntimeUiActionRequestQueue::default();

        let revoked = queue.record_result(0, &tree_id, &result);
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert!(drained.is_empty());
        assert_eq!(revoked.len(), 1);
    }

    #[test]
    fn redacted_secure_action_keeps_only_opaque_reference_and_route() {
        let tree_id = UiTreeId::new("queue.secure.redacted");
        let target = UiNodeId::new(7);
        let mut result = action_result(1);
        let reference = UiSecureTextValueRef::issue(tree_id.clone(), target, "value");
        result.component_events[0].event = UiComponentEvent::SecureCommit {
            property: "value".to_string(),
            reference: reference.clone(),
        };
        result.component_events[0]
            .template_action
            .as_mut()
            .unwrap()
            .payload
            .insert("credential".to_string(), UiValue::Null);
        let mut queue = RuntimeUiActionRequestQueue::default();

        let revoked = queue.record_result(0, &tree_id, &result);
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert_eq!(drained.len(), 1);
        assert!(revoked.is_empty());
        assert_eq!(drained[0].secure_value, Some(reference));
        assert_eq!(drained[0].invocation.target_id(), "runtime.test.action");
        let encoded = serde_json::to_string(&drained[0]).unwrap();
        assert!(!encoded.contains("must-not-cross-boundary"));
    }

    #[test]
    fn latest_secure_change_supersedes_only_the_same_pending_route() {
        let tree_id = UiTreeId::new("queue.secure.supersession");
        let target = UiNodeId::new(7);
        let mut first = action_result(1);
        first.component_events[0].event = UiComponentEvent::SecureValueChanged {
            property: "value".to_string(),
            reference: UiSecureTextValueRef::issue(tree_id.clone(), target, "value"),
        };
        let mut second = action_result(2);
        let latest = UiSecureTextValueRef::issue(tree_id.clone(), target, "value");
        second.component_events[0].event = UiComponentEvent::SecureValueChanged {
            property: "value".to_string(),
            reference: latest.clone(),
        };
        let mut queue = RuntimeUiActionRequestQueue::default();

        assert!(queue.record_result(0, &tree_id, &first).is_empty());
        assert!(queue.record_result(0, &tree_id, &second).is_empty());
        let mut drained = Vec::new();
        queue.drain_into(&mut drained);

        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].input_sequence, 2);
        assert_eq!(drained[0].secure_value, Some(latest));
    }

    fn action_result(sequence: u64) -> UiInputDispatchResult {
        let mut result = UiInputDispatchResult::new(
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(sequence),
                    UiInputSequence::new(sequence),
                ),
                text: String::new(),
            }),
            UiDispatchReply::handled(),
        );
        result.component_events.push(UiComponentEventReport {
            target: zircon_runtime_interface::ui::event_ui::UiNodeId::new(7),
            event: UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            },
            delivered: true,
            drag: None,
            template_action: Some(UiTemplateActionInvocation::route(
                "runtime.test.action",
                BTreeMap::new(),
            )),
        });
        result
    }
}
