use zircon_runtime_interface::ui::{
    dispatch::{UiInputDiagnosticsTruncationReceipt, UiInputDispatchResult},
    event_ui::UiNodeId,
};

pub(super) const MAX_ROUTE_NODES_PER_PATH: usize = 128;
const MAX_ROUTE_STEPS: usize = 256;
const MAX_NOTES: usize = 32;
const MAX_POPUP_ENTRIES: usize = 16;
const MAX_STRING_BYTES: usize = 8 * 1024;

pub(super) fn bounded_node_path(
    nodes: impl ExactSizeIterator<Item = UiNodeId>,
    truncation: &mut UiInputDiagnosticsTruncationReceipt,
) -> Vec<UiNodeId> {
    let retained = nodes.len().min(MAX_ROUTE_NODES_PER_PATH);
    record_dropped(
        &mut truncation.route_nodes_dropped,
        nodes.len().saturating_sub(retained),
    );
    let mut path = Vec::with_capacity(retained);
    path.extend(nodes.take(retained));
    path
}

pub(super) fn bounded_popup_stack<'popup>(
    popup_ids: impl ExactSizeIterator<Item = &'popup str>,
    truncation: &mut UiInputDiagnosticsTruncationReceipt,
) -> Vec<String> {
    let retained = popup_ids.len().min(MAX_POPUP_ENTRIES);
    let skipped = popup_ids.len().saturating_sub(retained);
    record_dropped(&mut truncation.popup_entries_dropped, skipped);

    let mut popup_stack = Vec::with_capacity(retained);
    for popup_id in popup_ids.skip(skipped) {
        popup_stack.push(bounded_string_copy(popup_id, truncation));
    }
    popup_stack
}

pub(super) fn diagnostics_budget_required(result: &UiInputDispatchResult) -> bool {
    let diagnostics = &result.diagnostics;
    let trace = &diagnostics.route_trace;
    diagnostics.handled_phase.is_some()
        || !trace.preview_tunnel.is_empty()
        || !trace.bubble_path.is_empty()
        || !trace.focus_path.is_empty()
        || !trace.root_targets.is_empty()
        || !trace.popup_stack.is_empty()
        || !diagnostics.route_steps.is_empty()
        || !diagnostics.notes.is_empty()
}

pub(super) fn enforce_diagnostics_budget(result: &mut UiInputDispatchResult) {
    let diagnostics = &mut result.diagnostics;
    truncate_vec(
        &mut diagnostics.route_trace.preview_tunnel,
        MAX_ROUTE_NODES_PER_PATH,
        &mut diagnostics.truncation.route_nodes_dropped,
    );
    truncate_vec(
        &mut diagnostics.route_trace.bubble_path,
        MAX_ROUTE_NODES_PER_PATH,
        &mut diagnostics.truncation.route_nodes_dropped,
    );
    truncate_vec(
        &mut diagnostics.route_trace.focus_path,
        MAX_ROUTE_NODES_PER_PATH,
        &mut diagnostics.truncation.route_nodes_dropped,
    );
    truncate_vec(
        &mut diagnostics.route_trace.root_targets,
        MAX_ROUTE_NODES_PER_PATH,
        &mut diagnostics.truncation.route_nodes_dropped,
    );
    truncate_vec(
        &mut diagnostics.route_steps,
        MAX_ROUTE_STEPS,
        &mut diagnostics.truncation.route_steps_dropped,
    );
    truncate_strings(
        &mut diagnostics.notes,
        MAX_NOTES,
        &mut diagnostics.truncation.notes_dropped,
        &mut diagnostics.truncation.string_bytes_dropped,
    );
    truncate_strings(
        &mut diagnostics.route_trace.popup_stack,
        MAX_POPUP_ENTRIES,
        &mut diagnostics.truncation.popup_entries_dropped,
        &mut diagnostics.truncation.string_bytes_dropped,
    );

    let mut remaining_string_bytes = MAX_STRING_BYTES;
    if let Some(handled_phase) = diagnostics.handled_phase.as_mut() {
        truncate_string_to_remaining(
            handled_phase,
            &mut remaining_string_bytes,
            &mut diagnostics.truncation.string_bytes_dropped,
        );
    }
    for popup_id in &mut diagnostics.route_trace.popup_stack {
        truncate_string_to_remaining(
            popup_id,
            &mut remaining_string_bytes,
            &mut diagnostics.truncation.string_bytes_dropped,
        );
    }
    for note in &mut diagnostics.notes {
        truncate_string_to_remaining(
            note,
            &mut remaining_string_bytes,
            &mut diagnostics.truncation.string_bytes_dropped,
        );
    }
}

fn bounded_string_copy(
    value: &str,
    truncation: &mut UiInputDiagnosticsTruncationReceipt,
) -> String {
    let retained = utf8_prefix_len(value, MAX_STRING_BYTES);
    record_dropped(
        &mut truncation.string_bytes_dropped,
        value.len().saturating_sub(retained),
    );
    value[..retained].to_string()
}

fn truncate_vec<T>(values: &mut Vec<T>, limit: usize, dropped: &mut u64) {
    record_dropped(dropped, values.len().saturating_sub(limit));
    values.truncate(limit);
}

fn truncate_strings(
    values: &mut Vec<String>,
    limit: usize,
    dropped_entries: &mut u64,
    dropped_bytes: &mut u64,
) {
    if values.len() <= limit {
        return;
    }
    record_dropped(dropped_entries, values.len() - limit);
    for value in &values[limit..] {
        record_dropped(dropped_bytes, value.len());
    }
    values.truncate(limit);
}

fn truncate_string_to_remaining(value: &mut String, remaining: &mut usize, dropped: &mut u64) {
    let retained = utf8_prefix_len(value, *remaining);
    record_dropped(dropped, value.len().saturating_sub(retained));
    value.truncate(retained);
    *remaining = (*remaining).saturating_sub(retained);
}

fn utf8_prefix_len(value: &str, limit: usize) -> usize {
    let mut retained = value.len().min(limit);
    while !value.is_char_boundary(retained) {
        retained -= 1;
    }
    retained
}

fn record_dropped(counter: &mut u64, dropped: usize) {
    *counter = counter.saturating_add(u64::try_from(dropped).unwrap_or(u64::MAX));
}

#[cfg(test)]
mod tests {
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
        result.diagnostics.route_trace.popup_stack =
            vec!["popup".to_string(); MAX_POPUP_ENTRIES + 2];
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
}
