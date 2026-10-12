use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_editor_binding;

#[cfg(test)]
pub(crate) fn dispatch_inspector_delete_selected(
    runtime: &EditorHostEventController,
) -> Result<UiHostEventEffects, String> {
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "InspectorView",
            "DeleteSelected",
            crate::ui::binding::EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action("workbench.selection.delete_selected"),
        ),
    )
}
