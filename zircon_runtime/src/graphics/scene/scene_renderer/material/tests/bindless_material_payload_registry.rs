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
    let production = include_str!("../bindless_material_payload_registry.rs")
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
    println!("RUNTIME553_SINGLE_SLOT_LOOKUP_BENCH_V1 legacy={legacy:?} optimized={optimized:?}");
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
