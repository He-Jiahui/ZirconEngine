use zircon_runtime_interface::ui::{
    dispatch::{UiInputDiagnosticsTruncationReceipt, UiInputDispatchResult},
    event_ui::UiNodeId,
};

pub(super) const MAX_ROUTE_NODES_PER_PATH: usize = 128;
const MAX_ROUTE_STEPS: usize = 256;
const MAX_NOTES: usize = 32;
const MAX_POPUP_ENTRIES: usize = 16;
const MAX_STRING_BYTES: usize = 8 * 1024;

/// 为路由诊断保留有界前缀，累计被省略的身份数，避免深树产生无界结果。
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

/// 弹层诊断保留栈顶尾部，因为最上层弹层决定当前输入拦截权。
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

/// 在结果出 surface 前统一限制集合和 UTF-8 字节量，并记录截断回执。
/// 预算只作用于诊断副本，不应修改回复的 effect、宿主请求或组件事件。
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
#[path = "tests/diagnostics_budget.rs"]
mod tests;
