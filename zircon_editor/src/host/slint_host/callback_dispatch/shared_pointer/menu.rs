use zircon_ui::UiPoint;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    event_bridge::SlintDispatchEffects,
    menu_pointer::{WorkbenchMenuPointerBridge, WorkbenchMenuPointerDispatch},
};

use super::super::{
    BuiltinWorkbenchTemplateBridge, dispatch_workbench_menu_action_with_template_fallback,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedMenuPointerClickDispatch {
    pub pointer: WorkbenchMenuPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_menu_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchMenuPointerBridge,
    point: UiPoint,
) -> Result<SharedMenuPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = pointer
        .action_id
        .as_deref()
        .map(|action| {
            dispatch_workbench_menu_action_with_template_fallback(runtime, template_bridge, action)
        })
        .transpose()?;
    Ok(SharedMenuPointerClickDispatch { pointer, effects })
}
