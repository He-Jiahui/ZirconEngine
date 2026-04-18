use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinWorkbenchTemplateBridge};

pub(crate) fn dispatch_builtin_workbench_host_page_activation(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    page_id: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.host_page_activation_binding(page_id)?;
    Some(dispatch_editor_binding(runtime, binding))
}
