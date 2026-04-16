use zircon_ui::UiPoint;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    event_bridge::SlintDispatchEffects,
    hierarchy_pointer::{HierarchyPointerBridge, HierarchyPointerDispatch, HierarchyPointerRoute},
};

use super::super::dispatch_hierarchy_selection;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedHierarchyPointerClickDispatch {
    pub pointer: HierarchyPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_hierarchy_pointer_click(
    runtime: &EditorEventRuntime,
    pointer_bridge: &mut HierarchyPointerBridge,
    point: UiPoint,
) -> Result<SharedHierarchyPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point)?;
    let effects = match pointer.route.as_ref() {
        Some(HierarchyPointerRoute::Node { node_id, .. }) => {
            let node_id = node_id
                .parse()
                .map_err(|error| format!("Invalid node id {node_id}: {error}"))?;
            Some(dispatch_hierarchy_selection(runtime, node_id)?)
        }
        _ => None,
    };

    Ok(SharedHierarchyPointerClickDispatch { pointer, effects })
}
