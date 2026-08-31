use zircon_runtime_interface::ui::layout::UiPoint;

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    activity_rail_pointer::{
        HostActivityRailPointerBridge, HostActivityRailPointerDispatch,
        HostActivityRailPointerRoute, HostActivityRailPointerSide,
    },
    event_bridge::UiHostEventEffects,
};

use super::super::dispatch_builtin_host_drawer_toggle;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedActivityRailPointerClickDispatch {
    pub pointer: HostActivityRailPointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_activity_rail_pointer_click(
    runtime: &EditorHostEventController,
    pointer_bridge: &mut HostActivityRailPointerBridge,
    side: HostActivityRailPointerSide,
    point: UiPoint,
) -> Result<SharedActivityRailPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(side, point)?;
    let effects = match pointer.route {
        Some(HostActivityRailPointerRoute::Button {
            side, item_index, ..
        }) => pointer_bridge
            .target_for_button(side, item_index)
            .map(|(slot, instance_id)| {
                dispatch_builtin_host_drawer_toggle(runtime, slot, instance_id)
            })
            .transpose()?,
        _ => None,
    };
    Ok(SharedActivityRailPointerClickDispatch { pointer, effects })
}
