use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{BuiltinWorkbenchTemplateBridge, common::dispatch_editor_binding};

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
