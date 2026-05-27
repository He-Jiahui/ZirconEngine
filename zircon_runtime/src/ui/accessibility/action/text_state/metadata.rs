use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::{UiNodeId, UiReflectedPropertySource},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::super::result::append_binding_report_diagnostic;

pub(super) fn clear_text_input_composition_metadata(
    surface: &mut UiSurface,
    target: UiNodeId,
    collapse_offset: usize,
    result: &mut UiInputDispatchResult,
) {
    let offset = i64::try_from(collapse_offset).unwrap_or(i64::MAX);
    for (property, value) in [
        ("composition_start", UiValue::Int(offset)),
        ("composition_end", UiValue::Int(offset)),
        ("composition_text", UiValue::String(String::new())),
        ("composition_restore_text", UiValue::String(String::new())),
    ] {
        mutate_text_input_accessibility_metadata(
            surface,
            target,
            property,
            value,
            "accessibility_text_composition",
            result,
        );
    }
}

pub(super) fn mutate_text_input_accessibility_metadata(
    surface: &mut UiSurface,
    target: UiNodeId,
    property: &str,
    value: UiValue,
    note_prefix: &str,
    result: &mut UiInputDispatchResult,
) {
    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(target, property, value)
            .with_source(UiReflectedPropertySource::RuntimeState),
    );
    match report {
        Ok(report) => {
            let status = report.status;
            let property = report.property.clone();
            append_binding_report_diagnostic(result, &report);
            match status {
                UiPropertyMutationStatus::Accepted => result
                    .diagnostics
                    .notes
                    .push(format!("{note_prefix}_changed:{property}")),
                UiPropertyMutationStatus::Unchanged => result
                    .diagnostics
                    .notes
                    .push(format!("{note_prefix}_unchanged:{property}")),
                UiPropertyMutationStatus::Rejected => result
                    .diagnostics
                    .notes
                    .push(format!("{note_prefix}_rejected:{property}")),
            }
        }
        Err(error) => result
            .diagnostics
            .notes
            .push(format!("{note_prefix}_error:{property}:{error}")),
    }
}
