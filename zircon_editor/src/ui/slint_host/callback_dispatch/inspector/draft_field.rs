use crate::ui::{
    inspector_field_control_id, DraftCommand, EditorUiBinding, EditorUiBindingPayload,
};
use zircon_ui::UiBindingValue;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_editor_binding;

#[cfg(test)]
pub(crate) fn dispatch_inspector_draft_field(
    runtime: &EditorEventRuntime,
    subject_path: impl Into<String>,
    field_id: impl Into<String>,
    value: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    let field_id = field_id.into();
    let control_id = inspector_field_control_id(field_id.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| field_id.clone());
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "InspectorView",
            control_id,
            crate::ui::EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                subject_path: subject_path.into(),
                field_id,
                value: UiBindingValue::string(value.into()),
            }),
        ),
    )
}
