use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{component::UiDragMetrics, event_ui::UiNodeId, layout::UiPoint};

use super::UiSurfaceInputState;

const POINTER_DRAG_HASH_CLEAR_THRESHOLD: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePointerDragResizeState {
    pub field: Arc<str>,
    pub start_width: f64,
    pub min_width: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePointerDragState {
    pub start: UiPoint,
    pub current: UiPoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resize: Option<UiSurfacePointerDragResizeState>,
}

impl UiSurfaceInputState {
    pub fn begin_pointer_drag(&mut self, owner: UiNodeId, point: UiPoint) -> UiDragMetrics {
        self.begin_pointer_drag_with_property(owner, point, None)
    }

    pub fn begin_pointer_drag_with_property(
        &mut self,
        owner: UiNodeId,
        point: UiPoint,
        property: Option<String>,
    ) -> UiDragMetrics {
        self.pointer_drags.insert(
            owner,
            UiSurfacePointerDragState {
                start: point,
                current: point,
                property,
                resize: None,
            },
        );
        UiDragMetrics::begin(point)
    }

    pub fn begin_pointer_drag_with_resize(
        &mut self,
        owner: UiNodeId,
        point: UiPoint,
        field: impl AsRef<str>,
        start_width: f64,
        min_width: f64,
    ) -> UiDragMetrics {
        self.pointer_drags.insert(
            owner,
            UiSurfacePointerDragState {
                start: point,
                current: point,
                property: None,
                resize: Some(UiSurfacePointerDragResizeState {
                    field: Arc::from(field.as_ref()),
                    start_width,
                    min_width,
                }),
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
                property: None,
                resize: None,
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
                property: None,
                resize: None,
            });
        UiDragMetrics::end(drag.start, point)
    }

    pub fn pointer_drag_property(&self, owner: UiNodeId) -> Option<&str> {
        self.pointer_drags
            .get(&owner)
            .and_then(|drag| drag.property.as_deref())
    }

    pub fn pointer_drag_resize(&self, owner: UiNodeId) -> Option<&UiSurfacePointerDragResizeState> {
        self.pointer_drags
            .get(&owner)
            .and_then(|drag| drag.resize.as_ref())
    }

    pub fn set_pointer_drag_property(&mut self, owner: UiNodeId, property: Option<String>) {
        if let Some(drag) = self.pointer_drags.get_mut(&owner) {
            drag.property = property;
        }
    }

    pub fn clear_pointer_drag_for(&mut self, owner: UiNodeId) {
        self.pointer_drags.remove(&owner);
    }

    /// 清除指定节点关联的拖拽状态，供节点失效后的清理调用；达到阈值后使用哈希集合进行成员查找。
    pub fn clear_pointer_drags_for_nodes(&mut self, node_ids: &[UiNodeId]) {
        if node_ids.len() >= POINTER_DRAG_HASH_CLEAR_THRESHOLD {
            let node_ids = node_ids.iter().copied().collect::<HashSet<_>>();
            self.pointer_drags
                .retain(|owner, _| !node_ids.contains(owner));
            return;
        }
        self.pointer_drags
            .retain(|owner, _| !node_ids.contains(owner));
    }
}

#[cfg(test)]
#[path = "pointer_drag/tests/hash_clear_tests.rs"]
mod hash_clear_tests;
