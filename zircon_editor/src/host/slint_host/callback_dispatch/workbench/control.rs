use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinWorkbenchTemplateBridge};

pub(crate) fn dispatch_builtin_workbench_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    control_id: &str,
    event_kind: zircon_ui::UiEventKind,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.binding_for_control(control_id, event_kind)?.clone();
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_workbench_menu_action(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinWorkbenchTemplateBridge,
    action: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    dispatch_builtin_workbench_control(runtime, bridge, action, zircon_ui::UiEventKind::Click)
}
