use zircon_runtime_interface::ui::layout::UiPoint;

use super::{HierarchyPointerBridge, HierarchyPointerRoute};
use crate::ui::retained_host::input_policy::DRAG_START_DISTANCE_PX;

pub(super) struct HierarchyReparentDrag {
    origin: UiPoint,
    dragging: bool,
}

impl HierarchyPointerBridge {
    pub(crate) fn begin_reparent_drag(&mut self, point: UiPoint) {
        self.reparent_drag = matches!(
            self.route_at_point(point),
            Some(HierarchyPointerRoute::Node { .. })
        )
        .then_some(HierarchyReparentDrag {
            origin: point,
            dragging: false,
        });
    }

    pub(crate) fn observe_reparent_drag_move(&mut self, point: UiPoint) {
        if self.reparent_drag.is_none() {
            return;
        }
        if self.route_at_point(point).is_none() {
            self.cancel_reparent_drag();
            return;
        }
        let Some(drag) = self.reparent_drag.as_mut() else {
            return;
        };
        let dx = point.x - drag.origin.x;
        let dy = point.y - drag.origin.y;
        drag.dragging |= dx * dx + dy * dy >= DRAG_START_DISTANCE_PX * DRAG_START_DISTANCE_PX;
    }

    pub(crate) fn finish_reparent_drag(&mut self) -> bool {
        self.reparent_drag.take().is_some_and(|drag| drag.dragging)
    }

    pub(crate) fn cancel_reparent_drag(&mut self) {
        self.reparent_drag = None;
    }
}

#[cfg(test)]
#[path = "tests/gesture.rs"]
mod tests;
