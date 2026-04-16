use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_editor_binding;

#[cfg(test)]
pub(crate) fn dispatch_inspector_delete_selected(
    runtime: &EditorEventRuntime,
) -> Result<SlintDispatchEffects, String> {
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "InspectorView",
            "DeleteSelected",
            zircon_editor_ui::EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action("DeleteSelected"),
        ),
    )
}
