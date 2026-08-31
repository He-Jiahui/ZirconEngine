use std::fmt::Write as _;

use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    dispatch::{UiDispatchReply, UiInputDispatchResult},
    event_ui::UiNodeId,
};

use crate::ui::surface::UiPropertyMutationReport;

mod binding;

pub(in crate::ui::accessibility::action) fn append_binding_report_diagnostic(
    result: &mut UiInputDispatchResult,
    report: &UiPropertyMutationReport,
) {
    binding::append_binding_report_diagnostic(result, report);
}

pub(super) fn unsupported_role_action(
    result: UiInputDispatchResult,
    target: UiNodeId,
    reason: &str,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Unsupported,
        "unsupported_role_action",
        reason,
    )
}

pub(super) fn finish_handled(
    mut result: UiInputDispatchResult,
    target: UiNodeId,
    phase: &str,
) -> UiInputDispatchResult {
    result.reply = UiDispatchReply::handled().from_handler(target);
    result.diagnostics.routed = true;
    result.diagnostics.route_target = Some(target);
    result.diagnostics.handled_phase = Some(phase.to_string());
    result.diagnostics.notes.push(action_note(
        UiAccessibilityActionStatus::Accepted,
        None,
        None,
    ));
    result
}

pub(super) fn finish_unhandled(
    mut result: UiInputDispatchResult,
    route_target: Option<UiNodeId>,
    status: UiAccessibilityActionStatus,
    code: &str,
    reason: &str,
) -> UiInputDispatchResult {
    result.diagnostics.route_target = route_target;
    result
        .diagnostics
        .notes
        .push(action_note(status, Some(code), Some(reason)));
    result
}

pub(super) fn finish_unhandled_with_note(
    result: UiInputDispatchResult,
    route_target: Option<UiNodeId>,
    status: UiAccessibilityActionStatus,
    code: &str,
    reason: &str,
    note: &str,
) -> UiInputDispatchResult {
    let mut result = finish_unhandled(result, route_target, status, code, reason);
    result.diagnostics.notes.push(note.to_string());
    result
}

pub(super) fn action_note(
    status: UiAccessibilityActionStatus,
    code: Option<&str>,
    reason: Option<&str>,
) -> String {
    let status = status_label(status);
    let capacity = "status=".len()
        + status.len()
        + code.map_or(0, |code| " code=".len() + code.len())
        + reason.map_or(0, |reason| " reason=".len() + reason.len());
    let mut note = String::with_capacity(capacity);
    write!(note, "status={status}").expect("writing to a string cannot fail");
    if let Some(code) = code {
        write!(note, " code={code}").expect("writing to a string cannot fail");
    }
    if let Some(reason) = reason {
        write!(note, " reason={reason}").expect("writing to a string cannot fail");
    }
    note
}

#[cfg(test)]
#[path = "result/single_buffer_action_note_tests.rs"]
mod single_buffer_action_note_tests;

fn status_label(status: UiAccessibilityActionStatus) -> &'static str {
    match status {
        UiAccessibilityActionStatus::Accepted => "accepted",
        UiAccessibilityActionStatus::Rejected => "rejected",
        UiAccessibilityActionStatus::Unsupported => "unsupported",
        UiAccessibilityActionStatus::StaleTarget => "stale_target",
    }
}
