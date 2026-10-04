use super::{
    UiInputDiagnosticsTruncationReceipt, UiInputDispatchDiagnostics, UiInputDispatchResult,
    UiPointerRoutingReceipt, UiTextInputConstraintReceipt,
};
use crate::ui::dispatch::{
    UiClipboardTransferId, UiClipboardTransferIntent, UiClipboardTransferReceipt,
    UiClipboardTransferStatus, UiDispatchReply, UiInputEvent, UiInputEventMetadata,
    UiMouseMotionInputEvent, UiNumberInputCommitMethod, UiNumberInputCommitStatus,
    UiNumberInputParseStatus, UiNumberInputReceiptV1,
};
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::{UiHitPath, UiPointerRoutingPath};

#[test]
fn pointer_routing_receipt_reuses_physical_path_until_dispatch_is_redirected() {
    let physical_hit_path = UiHitPath {
        target: Some(UiNodeId::new(3)),
        root_to_leaf: vec![UiNodeId::new(1), UiNodeId::new(3)],
        virtual_pointer: None,
    };
    let ordinary = UiPointerRoutingReceipt {
        route_target: Some(UiNodeId::new(3)),
        capture_target: None,
        physical_hit_path: physical_hit_path.clone(),
        dispatch_path: UiPointerRoutingPath::HitPath,
    };

    assert_eq!(
        ordinary.dispatch_root_to_leaf(),
        ordinary.physical_root_to_leaf()
    );
    assert_eq!(
        ordinary.dispatch_bubble_route().collect::<Vec<_>>(),
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );

    let captured = UiPointerRoutingReceipt {
        route_target: Some(UiNodeId::new(2)),
        capture_target: Some(UiNodeId::new(2)),
        physical_hit_path,
        dispatch_path: UiPointerRoutingPath::from_root_to_leaf(vec![
            UiNodeId::new(1),
            UiNodeId::new(2),
        ]),
    };
    assert_eq!(
        captured.physical_root_to_leaf(),
        &[UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(
        captured.dispatch_root_to_leaf(),
        &[UiNodeId::new(1), UiNodeId::new(2)]
    );

    let roundtrip: UiPointerRoutingReceipt =
        serde_json::from_value(serde_json::to_value(&captured).unwrap()).unwrap();
    assert_eq!(roundtrip, captured);
}

#[test]
fn pointer_routing_receipt_physical_bubble_route_is_bidirectional_and_repeatable() {
    let receipt = UiPointerRoutingReceipt {
        physical_hit_path: UiHitPath {
            target: Some(UiNodeId::new(3)),
            root_to_leaf: vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)],
            virtual_pointer: None,
        },
        ..UiPointerRoutingReceipt::default()
    };
    let bubble_route = receipt.physical_bubble_route();
    let repeated_route = bubble_route.clone();

    assert_eq!(
        bubble_route.collect::<Vec<_>>(),
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        receipt.physical_bubble_route().rev().collect::<Vec<_>>(),
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        repeated_route.collect::<Vec<_>>(),
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );
}

#[test]
fn legacy_dispatch_result_without_pointer_receipt_defaults_to_none() {
    let result = UiInputDispatchResult::new(
        UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
            metadata: UiInputEventMetadata::default(),
            delta_x: 0.0,
            delta_y: 0.0,
        }),
        UiDispatchReply::unhandled(),
    );
    let mut json = serde_json::to_value(result).unwrap();
    json.as_object_mut().unwrap().remove("pointer_routing");

    let legacy: UiInputDispatchResult = serde_json::from_value(json).unwrap();
    assert!(legacy.pointer_routing.is_none());
}

#[test]
fn text_constraint_receipt_roundtrips_and_defaults_when_missing() {
    let expected = UiTextInputConstraintReceipt {
        removed_hard_line_count: 2,
        removed_filter_scalar_count: 3,
        max_graphemes_truncated: true,
        preedit_cursor_range_adjusted: true,
        preedit_clause_range_adjusted_count: 4,
        preedit_clause_dropped_count: 5,
    };
    let diagnostics = UiInputDispatchDiagnostics {
        text_constraint: Some(expected),
        number_input: Some(UiNumberInputReceiptV1 {
            parse_status: UiNumberInputParseStatus::Valid,
            commit_method: UiNumberInputCommitMethod::Enter,
            commit_status: UiNumberInputCommitStatus::Applied,
            ..UiNumberInputReceiptV1::default()
        }),
        clipboard_transfer: Some(UiClipboardTransferReceipt {
            transfer_id: UiClipboardTransferId::issue(),
            intent: Some(UiClipboardTransferIntent::Paste),
            status: UiClipboardTransferStatus::Applied,
        }),
        secure_text_redacted: true,
        ..UiInputDispatchDiagnostics::default()
    };

    let mut json = serde_json::to_value(&diagnostics).unwrap();
    let roundtrip: UiInputDispatchDiagnostics = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(roundtrip.text_constraint, Some(expected));
    assert_eq!(roundtrip.number_input, diagnostics.number_input);
    assert_eq!(roundtrip.clipboard_transfer, diagnostics.clipboard_transfer);
    assert!(roundtrip.secure_text_redacted);

    json.as_object_mut().unwrap().remove("text_constraint");
    json.as_object_mut().unwrap().remove("number_input");
    json.as_object_mut().unwrap().remove("clipboard_transfer");
    json.as_object_mut().unwrap().remove("secure_text_redacted");
    json.as_object_mut().unwrap().remove("truncation");
    let legacy: UiInputDispatchDiagnostics = serde_json::from_value(json).unwrap();
    assert!(legacy.text_constraint.is_none());
    assert!(legacy.number_input.is_none());
    assert!(legacy.clipboard_transfer.is_none());
    assert!(!legacy.secure_text_redacted);
    assert!(legacy.truncation.is_empty());

    let truncation = UiInputDiagnosticsTruncationReceipt {
        route_nodes_dropped: 1,
        route_steps_dropped: 2,
        notes_dropped: 3,
        popup_entries_dropped: 4,
        string_bytes_dropped: 5,
    };
    let roundtrip: UiInputDiagnosticsTruncationReceipt =
        serde_json::from_value(serde_json::to_value(truncation).unwrap()).unwrap();
    assert_eq!(roundtrip, truncation);
    assert!(!roundtrip.is_empty());
}
