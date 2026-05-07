use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_editor_binding;

#[cfg(test)]
pub(crate) fn dispatch_inspector_delete_selected(
    runtime: &EditorEventRuntime,
) -> Result<UiHostEventEffects, String> {
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "InspectorView",
            "DeleteSelected",
            crate::ui::binding::EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action("DeleteSelected"),
        ),
    )
}
