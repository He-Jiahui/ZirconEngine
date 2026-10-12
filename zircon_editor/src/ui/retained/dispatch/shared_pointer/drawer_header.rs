use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    drawer_header_pointer::{
        HostDrawerHeaderPointerBridge, HostDrawerHeaderPointerDispatch,
        HostDrawerHeaderPointerRoute,
    },
    event_bridge::UiHostEventEffects,
};

use super::super::dispatch_builtin_host_drawer_toggle;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedDrawerHeaderPointerClickDispatch {
    pub pointer: HostDrawerHeaderPointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_drawer_header_pointer_click(
    runtime: &EditorHostEventController,
    pointer_bridge: &HostDrawerHeaderPointerBridge,
    surface_key: &str,
    item_index: usize,
) -> Result<SharedDrawerHeaderPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(surface_key, item_index)?;
    let effects = match pointer.route {
        Some(route @ HostDrawerHeaderPointerRoute::Tab { .. }) => {
            let (slot, instance_id) = pointer_bridge
                .target_for_route(route)
                .ok_or_else(|| "Drawer header receipt target is stale".to_string())?;
            Some(dispatch_builtin_host_drawer_toggle(
                runtime,
                slot,
                instance_id,
            )?)
        }
        _ => None,
    };
    Ok(SharedDrawerHeaderPointerClickDispatch { pointer, effects })
}
