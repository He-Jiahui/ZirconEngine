use zircon_ui::UiPoint;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    activity_rail_pointer::{
        WorkbenchActivityRailPointerBridge, WorkbenchActivityRailPointerDispatch,
        WorkbenchActivityRailPointerRoute, WorkbenchActivityRailPointerSide,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{dispatch_builtin_workbench_drawer_toggle, BuiltinWorkbenchTemplateBridge};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedActivityRailPointerClickDispatch {
    pub pointer: WorkbenchActivityRailPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_activity_rail_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchActivityRailPointerBridge,
    side: WorkbenchActivityRailPointerSide,
    point: UiPoint,
) -> Result<SharedActivityRailPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(side, point)?;
    let effects = match pointer.route.as_ref() {
        Some(WorkbenchActivityRailPointerRoute::Button {
            slot, instance_id, ..
        }) => dispatch_builtin_workbench_drawer_toggle(runtime, template_bridge, slot, instance_id)
            .transpose()?,
        _ => None,
    };
    Ok(SharedActivityRailPointerClickDispatch { pointer, effects })
}
