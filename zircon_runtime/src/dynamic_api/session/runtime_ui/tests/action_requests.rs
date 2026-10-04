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

#[test]
fn action_queue_revocation_scratch_reserves_component_event_bound() {
    let source = include_str!("../action_requests.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("let mut revoked_secure_values = Vec::new();"));
    assert!(implementation.contains(
        "let remaining_events = result.component_events.len().saturating_sub(action_index);"
    ));
    assert!(implementation.contains("revoked_secure_values.reserve(remaining_events);"));
    assert!(implementation.contains("if revoked_secure_values.is_empty()"));
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
