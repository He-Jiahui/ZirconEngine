use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinHostWindowTemplateBridge};

pub(crate) fn dispatch_builtin_host_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    control_id: &str,
    event_kind: zircon_runtime::ui::binding::UiEventKind,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = bridge.binding_for_control(control_id, event_kind)?.clone();
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_host_menu_action(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    action: &str,
) -> Option<Result<SlintDispatchEffects, String>> {
    dispatch_builtin_host_control(
        runtime,
        bridge,
        action,
        zircon_runtime::ui::binding::UiEventKind::Click,
    )
}
