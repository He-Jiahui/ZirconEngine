use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiCanvasSlotPlacement, UiSlotKind},
    tree::{UiDirtyFlags, UiTreeError},
};

use super::{UiInvalidationReason, UiSurface};

impl UiSurface {
    /// 调整已有 Overlay 父子 slot 的堆叠顺序；只改变绘制与命中，不改节点自身 z_index。
    /// 返回是否发生变化，父子节点及指定类型的 slot 均须已存在。
    pub fn set_overlay_slot_z_order(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        z_order: i32,
    ) -> Result<bool, UiTreeError> {
        self.set_layering_slot_z_order(parent_id, child_id, UiSlotKind::Overlay, z_order)
    }

    /// 调整已有 Canvas slot 的堆叠顺序，供排布后的绘制和命中共同采用。
    /// 调用后由正常的脏帧重建发布结果，方法本身不重建布局。
    pub fn set_canvas_slot_z_order(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        z_order: i32,
    ) -> Result<bool, UiTreeError> {
        self.set_layering_slot_z_order(parent_id, child_id, UiSlotKind::Canvas, z_order)
    }

    /// 为已有 Free slot 更新相对父容器的放置约束；后续布局重建计算新的几何。
    pub fn set_free_slot_canvas_placement(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        placement: UiCanvasSlotPlacement,
    ) -> Result<bool, UiTreeError> {
        self.set_slot_canvas_placement(parent_id, child_id, UiSlotKind::Free, placement)
    }

    /// 为已有 Canvas slot 更新锚点、偏移和枢轴约束，并使该子节点布局失效。
    pub fn set_canvas_slot_canvas_placement(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        placement: UiCanvasSlotPlacement,
    ) -> Result<bool, UiTreeError> {
        self.set_slot_canvas_placement(parent_id, child_id, UiSlotKind::Canvas, placement)
    }

    fn set_slot_canvas_placement(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        slot_kind: UiSlotKind,
        placement: UiCanvasSlotPlacement,
    ) -> Result<bool, UiTreeError> {
        self.ensure_slot_endpoints(parent_id, child_id)?;
        let slot_index = self.slot_index(parent_id, child_id, slot_kind)?;
        if self.tree.layout_slots()[slot_index].canvas_placement == Some(placement) {
            return Ok(false);
        }

        let _ = self.tree.mutate_layout_slot(slot_index, |slot| {
            slot.canvas_placement = Some(placement);
            slot.dirty_revision = slot.dirty_revision.saturating_add(1);
        });
        self.invalidate_node(child_id, UiInvalidationReason::Layout)?;
        Ok(true)
    }

    fn set_layering_slot_z_order(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        slot_kind: UiSlotKind,
        z_order: i32,
    ) -> Result<bool, UiTreeError> {
        self.ensure_slot_endpoints(parent_id, child_id)?;
        let slot_index = self.slot_index(parent_id, child_id, slot_kind)?;
        if self.tree.layout_slots()[slot_index].z_order == z_order {
            return Ok(false);
        }

        let _ = self.tree.mutate_layout_slot(slot_index, |slot| {
            slot.z_order = z_order;
            slot.dirty_revision = slot.dirty_revision.saturating_add(1);
        });
        let dirty = layering_slot_z_order_dirty_flags();
        self.mark_node_dirty(child_id, dirty)?;
        self.invalidation.record_dirty_with_reason(
            child_id,
            dirty,
            UiInvalidationReason::Structure,
        );
        Ok(true)
    }

    fn ensure_slot_endpoints(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> Result<(), UiTreeError> {
        if !self.tree.nodes.contains_key(&parent_id) {
            return Err(UiTreeError::MissingParent(parent_id));
        }
        if !self.tree.nodes.contains_key(&child_id) {
            return Err(UiTreeError::MissingNode(child_id));
        }
        Ok(())
    }

    fn slot_index(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: UiSlotKind,
    ) -> Result<usize, UiTreeError> {
        self.layout_slot_index
            .index_for_kind(&self.tree, parent_id, child_id, kind)
            .ok_or(UiTreeError::MissingNode(child_id))
    }
}

fn layering_slot_z_order_dirty_flags() -> UiDirtyFlags {
    UiDirtyFlags {
        hit_test: true,
        render: true,
        ..UiDirtyFlags::default()
    }
}
