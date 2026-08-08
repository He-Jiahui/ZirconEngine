use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiCanvasSlotPlacement, UiSlotKind},
    tree::{UiDirtyFlags, UiTreeError},
};

use super::{UiInvalidationReason, UiSurface};

impl UiSurface {
    pub fn set_overlay_slot_z_order(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        z_order: i32,
    ) -> Result<bool, UiTreeError> {
        self.set_layering_slot_z_order(parent_id, child_id, UiSlotKind::Overlay, z_order)
    }

    pub fn set_canvas_slot_z_order(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        z_order: i32,
    ) -> Result<bool, UiTreeError> {
        self.set_layering_slot_z_order(parent_id, child_id, UiSlotKind::Canvas, z_order)
    }

    pub fn set_free_slot_canvas_placement(
        &mut self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        placement: UiCanvasSlotPlacement,
    ) -> Result<bool, UiTreeError> {
        self.set_slot_canvas_placement(parent_id, child_id, UiSlotKind::Free, placement)
    }

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
        if self.tree.slots[slot_index].canvas_placement == Some(placement) {
            return Ok(false);
        }

        {
            let slot = &mut self.tree.slots[slot_index];
            slot.canvas_placement = Some(placement);
            slot.dirty_revision = slot.dirty_revision.saturating_add(1);
        }
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
        if self.tree.slots[slot_index].z_order == z_order {
            return Ok(false);
        }

        {
            let slot = &mut self.tree.slots[slot_index];
            slot.z_order = z_order;
            slot.dirty_revision = slot.dirty_revision.saturating_add(1);
        }
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
        self.tree
            .slots
            .iter()
            .position(|slot| {
                slot.parent_id == parent_id && slot.child_id == child_id && slot.kind == kind
            })
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
