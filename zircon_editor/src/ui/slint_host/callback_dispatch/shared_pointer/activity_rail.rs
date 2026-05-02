use zircon_runtime_interface::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    activity_rail_pointer::{
        HostActivityRailPointerBridge, HostActivityRailPointerDispatch,
        HostActivityRailPointerRoute, HostActivityRailPointerSide,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{dispatch_builtin_host_drawer_toggle, BuiltinHostWindowTemplateBridge};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedActivityRailPointerClickDispatch {
    pub pointer: HostActivityRailPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_activity_rail_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostActivityRailPointerBridge,
    side: HostActivityRailPointerSide,
    point: UiPoint,
) -> Result<SharedActivityRailPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(side, point)?;
    let effects = match pointer.route.as_ref() {
        Some(HostActivityRailPointerRoute::Button {
            slot, instance_id, ..
        }) => dispatch_builtin_host_drawer_toggle(runtime, template_bridge, slot, instance_id)
            .transpose()?,
        _ => None,
    };
    Ok(SharedActivityRailPointerClickDispatch { pointer, effects })
}
