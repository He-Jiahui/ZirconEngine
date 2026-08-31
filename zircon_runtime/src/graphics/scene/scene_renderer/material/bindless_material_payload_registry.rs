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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::resource::ResourceId;

    use super::{BindlessMaterialPayloadRegistry, BindlessMaterialPayloadSlot};
    use crate::graphics::scene::scene_renderer::material::GpuBindlessMaterialPayload;

    fn resource(value: u32) -> ResourceId {
        ResourceId::from_stable_label(&format!("bindless-material-payload-test-{value}"))
    }

    fn payload(value: u32) -> GpuBindlessMaterialPayload {
        let mut payload = GpuBindlessMaterialPayload::default();
        payload.texture_slots[0] = value;
        payload
    }

    #[test]
    fn render_bindless_material_payload_registry_reuses_a_material_row_without_frame_sweeps() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();

        let first = registry.upsert(resource(1), 4, payload(11));
        let duplicate = registry.upsert(resource(1), 4, payload(11));

        assert_eq!(first.slot, BindlessMaterialPayloadSlot(1));
        assert!(first.payload_changed);
        assert_eq!(duplicate.slot, first.slot);
        assert!(!duplicate.payload_changed);
        assert_eq!(registry.active_material_count(), 1);
        assert_eq!(registry.take_dirty_slots(), vec![first.slot]);

        let repeated_prepare = registry.upsert(resource(1), 4, payload(11));

        assert_eq!(repeated_prepare.slot, first.slot);
        assert!(!repeated_prepare.payload_changed);
        assert!(registry.take_dirty_slots().is_empty());
    }

    #[test]
    fn render_bindless_material_payload_registry_updates_in_place_when_a_revision_changes() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();
        let first = registry.upsert(resource(9), 1, payload(2));
        registry.take_dirty_slots();

        let revised = registry.upsert(resource(9), 2, payload(3));

        assert_eq!(revised.slot, first.slot);
        assert!(revised.payload_changed);
        assert_eq!(registry.payload(revised.slot), payload(3));
        assert_eq!(registry.take_dirty_slots(), vec![first.slot]);
    }

    #[test]
    fn optimization_batch_20260830es_runtime553_updates_payload_with_one_slot_lookup() {
        let production = include_str!("bindless_material_payload_registry.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let upsert = production
            .split("pub(crate) fn upsert")
            .nth(1)
            .and_then(|source| source.split("pub(crate) fn release").next())
            .expect("upsert source");

        assert_eq!(upsert.matches("self.slots[slot.get() as usize]").count(), 1);

        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();
        let first = registry.upsert(resource(10), 4, payload(1));
        registry.take_dirty_slots();

        let updated = registry.upsert(resource(10), 4, payload(9));

        assert_eq!(updated.slot, first.slot);
        assert!(updated.payload_changed);
        assert_eq!(registry.payload(first.slot), payload(9));
        assert_eq!(registry.take_dirty_slots(), vec![first.slot]);
    }

    #[test]
    #[ignore = "deterministic performance marker"]
    fn optimization_batch_20260830es_runtime553_single_slot_lookup_benchmark() {
        const UPDATE_COUNT: usize = 2_000_000;
        const SLOT_COUNT: usize = 4_096;
        const SAMPLES: usize = 9;
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let mut slots = vec![0_u64; SLOT_COUNT];
            let started = Instant::now();
            for update in 0..UPDATE_COUNT {
                let slot = update & (SLOT_COUNT - 1);
                let changed = slots[slot] != update as u64;
                if changed {
                    slots[slot] = update as u64;
                }
            }
            black_box(&slots);
            legacy_samples.push(started.elapsed());

            let mut slots = vec![0_u64; SLOT_COUNT];
            let started = Instant::now();
            for update in 0..UPDATE_COUNT {
                let slot = update & (SLOT_COUNT - 1);
                let state = &mut slots[slot];
                let changed = *state != update as u64;
                if changed {
                    *state = update as u64;
                }
            }
            black_box(&slots);
            optimized_samples.push(started.elapsed());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy = legacy_samples[SAMPLES / 2];
        let optimized = optimized_samples[SAMPLES / 2];
        println!(
            "RUNTIME553_SINGLE_SLOT_LOOKUP_BENCH_V1 legacy={legacy:?} optimized={optimized:?}"
        );
    }

    #[test]
    fn render_bindless_material_payload_registry_coalesces_repeated_dirty_row_updates() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();

        let first = registry.upsert(resource(11), 1, payload(1));
        registry.upsert(resource(11), 2, payload(2));
        registry.upsert(resource(11), 3, payload(3));

        assert_eq!(registry.take_dirty_slots(), vec![first.slot]);
        assert_eq!(registry.payload(first.slot), payload(3));
    }

    #[test]
    fn render_bindless_material_payload_registry_reuses_released_rows_only_after_frame_advance() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();
        let stale = registry.upsert(resource(7), 1, payload(17));
        registry.take_dirty_slots();

        assert!(registry.release(resource(7)));

        assert_eq!(registry.active_material_count(), 0);
        assert_eq!(
            registry.payload(stale.slot),
            registry.payload(registry.fallback_slot())
        );
        assert_eq!(registry.take_dirty_slots(), vec![stale.slot]);

        registry.advance_frame();
        let replacement = registry.upsert(resource(8), 1, payload(23));

        assert_eq!(replacement.slot, stale.slot);
        assert_eq!(registry.allocated_slot_count(), 2);
    }

    #[test]
    fn render_bindless_material_payload_registry_does_not_alias_a_released_row_within_the_frame() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();
        let released = registry.upsert(resource(7), 1, payload(17));
        registry.take_dirty_slots();

        assert!(registry.release(resource(7)));
        let same_frame = registry.upsert(resource(8), 1, payload(23));

        assert_ne!(same_frame.slot, released.slot);
        assert_eq!(
            registry.payload(released.slot),
            registry.payload(registry.fallback_slot())
        );
        assert_eq!(
            registry.take_dirty_slots(),
            vec![released.slot, same_frame.slot]
        );
    }

    #[test]
    fn render_bindless_material_payload_registry_ignores_unknown_explicit_releases() {
        let mut registry = BindlessMaterialPayloadRegistry::default();
        registry.take_dirty_slots();

        assert!(!registry.release(resource(99)));
        assert!(registry.take_dirty_slots().is_empty());
    }
}
