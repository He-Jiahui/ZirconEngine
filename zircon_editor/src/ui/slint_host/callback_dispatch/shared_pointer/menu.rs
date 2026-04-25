use zircon_runtime::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::SlintDispatchEffects,
    menu_pointer::{HostMenuPointerBridge, HostMenuPointerDispatch},
};

use super::super::{
    dispatch_host_menu_action_with_template_fallback, BuiltinHostWindowTemplateBridge,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedMenuPointerClickDispatch {
    pub pointer: HostMenuPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_menu_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostMenuPointerBridge,
    point: UiPoint,
) -> Result<SharedMenuPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = pointer
        .action_id
        .as_deref()
        .map(|action| {
            dispatch_host_menu_action_with_template_fallback(runtime, template_bridge, action)
        })
        .transpose()?;
    Ok(SharedMenuPointerClickDispatch { pointer, effects })
}
