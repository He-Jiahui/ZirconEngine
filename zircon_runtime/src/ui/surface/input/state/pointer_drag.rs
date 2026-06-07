use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{component::UiDragMetrics, event_ui::UiNodeId, layout::UiPoint};

use super::UiSurfaceInputState;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePointerDragState {
    pub start: UiPoint,
    pub current: UiPoint,
}

impl UiSurfaceInputState {
    pub fn begin_pointer_drag(&mut self, owner: UiNodeId, point: UiPoint) -> UiDragMetrics {
        self.pointer_drags.insert(
            owner,
            UiSurfacePointerDragState {
                start: point,
                current: point,
            },
        );
        UiDragMetrics::begin(point)
    }

    pub fn update_pointer_drag(&mut self, owner: UiNodeId, point: UiPoint) -> UiDragMetrics {
        let drag = self
            .pointer_drags
            .entry(owner)
            .or_insert(UiSurfacePointerDragState {
                start: point,
                current: point,
            });
        drag.current = point;
        UiDragMetrics::update(drag.start, drag.current)
    }

    pub fn end_pointer_drag(&mut self, owner: UiNodeId, point: UiPoint) -> UiDragMetrics {
        let drag = self
            .pointer_drags
            .remove(&owner)
            .unwrap_or(UiSurfacePointerDragState {
                start: point,
                current: point,
            });
        UiDragMetrics::end(drag.start, point)
    }

    pub fn clear_pointer_drag_for(&mut self, owner: UiNodeId) {
        self.pointer_drags.remove(&owner);
    }

    pub fn clear_pointer_drags_for_nodes(&mut self, node_ids: &[UiNodeId]) {
        self.pointer_drags
            .retain(|owner, _| !node_ids.contains(owner));
    }
}
