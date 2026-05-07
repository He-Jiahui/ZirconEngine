use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::UiHostEventEffects;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::super::{common::dispatch_editor_binding, BuiltinHostWindowTemplateBridge};

pub(crate) fn dispatch_builtin_host_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    control_id: &str,
    event_kind: UiEventKind,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = bridge.binding_for_control(control_id, event_kind)?.clone();
    Some(dispatch_editor_binding(runtime, binding))
}

pub(crate) fn dispatch_builtin_host_menu_action(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinHostWindowTemplateBridge,
    action: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    dispatch_builtin_host_control(runtime, bridge, action, UiEventKind::Click)
}
