use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinWorkbenchTemplateBridge};

pub(crate) fn dispatch_builtin_workbench_document_tab_activation(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    instance_id: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.document_tab_activation_binding(instance_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_workbench_document_tab_close(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    instance_id: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.document_tab_close_binding(instance_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}
