use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::UiHostEventEffects;

use super::super::{common::dispatch_editor_binding, BuiltinHostWindowTemplateBridge};

pub(crate) fn dispatch_builtin_host_document_tab_activation(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    instance_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = bridge.document_tab_activation_binding(instance_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_host_document_tab_close(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    instance_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = bridge.document_tab_close_binding(instance_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}
