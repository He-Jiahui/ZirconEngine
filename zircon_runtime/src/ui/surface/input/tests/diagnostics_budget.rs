use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchPhase, UiDispatchReply, UiDispatchReplyStepTrace,
        UiInputDiagnosticsTruncationReceipt, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiMouseMotionInputEvent,
    },
    event_ui::UiNodeId,
};

use super::{
    bounded_node_path, bounded_popup_stack, diagnostics_budget_required,
    enforce_diagnostics_budget, MAX_NOTES, MAX_POPUP_ENTRIES, MAX_ROUTE_NODES_PER_PATH,
    MAX_ROUTE_STEPS, MAX_STRING_BYTES,
};

#[test]
fn budget_requirement_ignores_scalar_summary_receipts() {
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
            metadata: UiInputEventMetadata::default(),
            delta_x: 0.0,
            delta_y: 0.0,
        }),
        UiDispatchReply::unhandled(),
    );
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(UiNodeId::new(7));

    assert!(!diagnostics_budget_required(&result));

    result.diagnostics.notes.push("full-only".to_string());
    assert!(diagnostics_budget_required(&result));
}

#[test]
fn bounded_node_path_records_every_omitted_identity() {
    let mut truncation = UiInputDiagnosticsTruncationReceipt::default();

    let path = bounded_node_path(
        (0..MAX_ROUTE_NODES_PER_PATH + 7).map(|index| UiNodeId::new(index as u64)),
        &mut truncation,
    );

    assert_eq!(path.len(), MAX_ROUTE_NODES_PER_PATH);
    assert_eq!(truncation.route_nodes_dropped, 7);
}

#[test]
fn bounded_popup_stack_preserves_the_topmost_tail() {
    let popup_ids = (0..MAX_POPUP_ENTRIES + 3)
        .map(|index| format!("popup-{index}"))
        .collect::<Vec<_>>();
    let mut truncation = UiInputDiagnosticsTruncationReceipt::default();

    let retained = bounded_popup_stack(popup_ids.iter().map(String::as_str), &mut truncation);

    assert_eq!(retained.len(), MAX_POPUP_ENTRIES);
    assert_eq!(retained.first().map(String::as_str), Some("popup-3"));
    let expected_last = format!("popup-{}", MAX_POPUP_ENTRIES + 2);
    assert_eq!(
        retained.last().map(String::as_str),
        Some(expected_last.as_str())
    );
    assert_eq!(truncation.popup_entries_dropped, 3);
}

#[test]
fn final_budget_bounds_all_diagnostic_collections_and_utf8_bytes() {
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
            metadata: UiInputEventMetadata::default(),
            delta_x: 0.0,
            delta_y: 0.0,
        }),
        UiDispatchReply::unhandled(),
    );
    let oversized_nodes = (0..MAX_ROUTE_NODES_PER_PATH + 5)
        .map(|index| UiNodeId::new(index as u64))
        .collect::<Vec<_>>();
    result.diagnostics.route_trace.preview_tunnel = oversized_nodes.clone();
    result.diagnostics.route_trace.bubble_path = oversized_nodes.clone();
    result.diagnostics.route_trace.focus_path = oversized_nodes.clone();
    result.diagnostics.route_trace.root_targets = oversized_nodes;
    let step = UiDispatchReplyStepTrace {
        phase: UiDispatchPhase::Bubble,
        target: None,
        handler: None,
        disposition: UiDispatchDisposition::Passthrough,
        effect_start: 0,
        effect_count: 0,
        ignored_effect_count: 0,
        stopped: false,
    };
    result.diagnostics.route_steps = vec![step; MAX_ROUTE_STEPS + 9];
    result.diagnostics.notes = vec!["note".to_string(); MAX_NOTES + 4];
    result.diagnostics.route_trace.popup_stack = vec!["popup".to_string(); MAX_POPUP_ENTRIES + 2];
    result.diagnostics.handled_phase = Some("界".repeat(MAX_STRING_BYTES));

    enforce_diagnostics_budget(&mut result);

    let diagnostics = &result.diagnostics;
    assert_eq!(
        diagnostics.route_trace.preview_tunnel.len(),
        MAX_ROUTE_NODES_PER_PATH
    );
    assert_eq!(diagnostics.route_steps.len(), MAX_ROUTE_STEPS);
    assert_eq!(diagnostics.notes.len(), MAX_NOTES);
    assert_eq!(diagnostics.route_trace.popup_stack.len(), MAX_POPUP_ENTRIES);
    let retained_string_bytes = diagnostics
        .handled_phase
        .iter()
        .map(String::len)
        .chain(diagnostics.route_trace.popup_stack.iter().map(String::len))
        .chain(diagnostics.notes.iter().map(String::len))
        .sum::<usize>();
    assert!(retained_string_bytes <= MAX_STRING_BYTES);
    assert_eq!(diagnostics.truncation.route_nodes_dropped, 20);
    assert_eq!(diagnostics.truncation.route_steps_dropped, 9);
    assert_eq!(diagnostics.truncation.notes_dropped, 4);
    assert_eq!(diagnostics.truncation.popup_entries_dropped, 2);
    assert!(diagnostics.truncation.string_bytes_dropped > 0);
}
