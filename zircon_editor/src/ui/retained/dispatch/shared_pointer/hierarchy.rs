use zircon_runtime::scene::{NodeId, WorldInspectionHierarchyRow};
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    event_bridge::UiHostEventEffects,
    hierarchy_pointer::{HierarchyPointerBridge, HierarchyPointerDispatch, HierarchyPointerRoute},
};

use super::super::dispatch_hierarchy_selection;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedHierarchyPointerClickDispatch {
    pub pointer: HierarchyPointerDispatch,
    pub selected_entity: Option<NodeId>,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_hierarchy_pointer_click(
    runtime: &EditorHostEventController,
    pointer_bridge: &mut HierarchyPointerBridge,
    scene_entries: &[WorldInspectionHierarchyRow],
    point: UiPoint,
) -> Result<SharedHierarchyPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point);
    let selected_entity = match pointer.route {
        Some(HierarchyPointerRoute::Node { item_index }) => {
            let entry = scene_entries.get(item_index).ok_or_else(|| {
                format!("Hierarchy pointer item index {item_index} is outside the committed rows")
            })?;
            Some(entry.entity)
        }
        _ => None,
    };
    let effects = match selected_entity {
        Some(entity) => Some(dispatch_hierarchy_selection(runtime, entity)?),
        None => None,
    };

    Ok(SharedHierarchyPointerClickDispatch {
        pointer,
        selected_entity,
        effects,
    })
}
