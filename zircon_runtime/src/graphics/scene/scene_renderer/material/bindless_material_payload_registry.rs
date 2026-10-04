use std::collections::{HashMap, HashSet};

use crate::core::resource::ResourceId;

use super::GpuBindlessMaterialPayload;

/// Shader-visible row index into the bindless material-payload storage buffer.
///
/// Row zero is permanently reserved as a deterministic fallback. Material resource rows begin
/// at one so a malformed primitive index cannot address uninitialized storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindlessMaterialPayloadSlot(u32);

impl BindlessMaterialPayloadSlot {
    pub(crate) const FALLBACK: Self = Self(0);

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindlessMaterialPayloadPrepareResult {
    pub(crate) slot: BindlessMaterialPayloadSlot,
    /// True when the associated GPU buffer row must be uploaded before it is consumed.
    pub(crate) payload_changed: bool,
}

#[derive(Clone, Copy, Debug)]
struct MaterialPayloadEntry {
    revision: u64,
    slot: BindlessMaterialPayloadSlot,
}

#[derive(Clone, Copy, Debug, Default)]
struct MaterialPayloadSlotState {
    resource: Option<ResourceId>,
    payload: GpuBindlessMaterialPayload,
}

/// CPU ownership and dirty tracking for bindless material payload rows.
///
/// Rows remain allocated until the owner reports a resource or scene-binding removal through
/// [`Self::release`]. Stable resource IDs retain their slots and revisions overwrite that same
/// row, avoiding both per-draw material uploads and per-frame full-registry liveness scans.
#[derive(Debug)]
// TODO: [CR-GRAPHICS-AUX-B-0001] 当前检索仅见声明、导出和测试，尚未找到实际 dirty 上传与帧推进调用链；
// 需先确认 bindless 启用入口，再核对提交失败的重试及 advance_frame 时机，勿将 CPU 测试视为 GPU 消费证据。
pub(crate) struct BindlessMaterialPayloadRegistry {
    entries: HashMap<ResourceId, MaterialPayloadEntry>,
    slots: Vec<MaterialPayloadSlotState>,
    free_slots: Vec<u32>,
    retired_slots: Vec<u32>,
    dirty_slots: Vec<BindlessMaterialPayloadSlot>,
    dirty_slot_set: HashSet<BindlessMaterialPayloadSlot>,
}

impl Default for BindlessMaterialPayloadRegistry {
    fn default() -> Self {
        Self::new(GpuBindlessMaterialPayload::default())
    }
}

impl BindlessMaterialPayloadRegistry {
    pub(crate) fn new(fallback_payload: GpuBindlessMaterialPayload) -> Self {
        Self {
            entries: HashMap::new(),
            slots: vec![MaterialPayloadSlotState {
                resource: None,
                payload: fallback_payload,
            }],
            free_slots: Vec::new(),
            retired_slots: Vec::new(),
            dirty_slots: vec![BindlessMaterialPayloadSlot::FALLBACK],
            dirty_slot_set: HashSet::from([BindlessMaterialPayloadSlot::FALLBACK]),
        }
    }

    /// Returns the stable payload row for `resource`, updating that row only when its logical
    /// revision or encoded data changed.
    pub(crate) fn upsert(
        &mut self,
        resource: ResourceId,
        revision: u64,
        payload: GpuBindlessMaterialPayload,
    ) -> BindlessMaterialPayloadPrepareResult {
        if let Some(entry) = self.entries.get_mut(&resource) {
            let slot = entry.slot;
            let payload_changed = {
                let state = &mut self.slots[slot.get() as usize];
                let payload_changed = entry.revision != revision || state.payload != payload;
                if payload_changed {
                    entry.revision = revision;
                    state.payload = payload;
                }
                payload_changed
            };
            if payload_changed {
                self.mark_dirty(slot);
            }
            return BindlessMaterialPayloadPrepareResult {
                slot,
                payload_changed,
            };
        }

        let slot = self.allocate_slot(resource, payload);
        self.entries
            .insert(resource, MaterialPayloadEntry { revision, slot });
        self.mark_dirty(slot);
        BindlessMaterialPayloadPrepareResult {
            slot,
            payload_changed: true,
        }
    }

    /// Reclaims one row after its resource or binding owner has been removed.
    ///
    /// Released rows are reset to the fallback payload and retired until [`Self::advance_frame`]
    /// runs after the current frame has submitted. The returned rows are included in
    /// [`Self::take_dirty_slots`] so a stale primitive can only observe fallback data during that
    /// frame, never a newly allocated material payload.
    pub(crate) fn release(&mut self, resource: ResourceId) -> bool {
        let fallback_payload = self.fallback_payload();
        let Some(entry) = self.entries.remove(&resource) else {
            return false;
        };
        let state = &mut self.slots[entry.slot.get() as usize];
        state.resource = None;
        state.payload = fallback_payload;
        self.retired_slots.push(entry.slot.get());
        self.mark_dirty(entry.slot);
        true
    }

    /// Makes rows released by the previously submitted frame available for reuse.
    ///
    /// This is proportional only to released rows, not to the live material registry. Callers
    /// must advance after submission and before preparing the next frame.
    pub(crate) fn advance_frame(&mut self) {
        self.free_slots.append(&mut self.retired_slots);
    }

    pub(crate) fn payload(&self, slot: BindlessMaterialPayloadSlot) -> GpuBindlessMaterialPayload {
        self.slots
            .get(slot.get() as usize)
            .map(|state| state.payload)
            .unwrap_or_else(|| self.fallback_payload())
    }

    pub(crate) fn fallback_slot(&self) -> BindlessMaterialPayloadSlot {
        BindlessMaterialPayloadSlot::FALLBACK
    }

    pub(crate) fn active_material_count(&self) -> u32 {
        self.entries.len().min(u32::MAX as usize) as u32
    }

    pub(crate) fn allocated_slot_count(&self) -> u32 {
        self.slots.len().min(u32::MAX as usize) as u32
    }

    pub(crate) fn take_dirty_slots(&mut self) -> Vec<BindlessMaterialPayloadSlot> {
        self.dirty_slot_set.clear();
        std::mem::take(&mut self.dirty_slots)
    }

    fn allocate_slot(
        &mut self,
        resource: ResourceId,
        payload: GpuBindlessMaterialPayload,
    ) -> BindlessMaterialPayloadSlot {
        if let Some(index) = self.free_slots.pop() {
            let slot = BindlessMaterialPayloadSlot(index);
            let state = &mut self.slots[index as usize];
            debug_assert!(state.resource.is_none());
            state.resource = Some(resource);
            state.payload = payload;
            return slot;
        }

        let index = u32::try_from(self.slots.len())
            .expect("bindless material payload slot count exceeded u32");
        self.slots.push(MaterialPayloadSlotState {
            resource: Some(resource),
            payload,
        });
        BindlessMaterialPayloadSlot(index)
    }

    fn fallback_payload(&self) -> GpuBindlessMaterialPayload {
        self.slots[BindlessMaterialPayloadSlot::FALLBACK.get() as usize].payload
    }

    fn mark_dirty(&mut self, slot: BindlessMaterialPayloadSlot) {
        if self.dirty_slot_set.insert(slot) {
            self.dirty_slots.push(slot);
        }
    }
}

#[cfg(test)]
#[path = "tests/bindless_material_payload_registry.rs"]
mod tests;
