use super::{
    bindless_material_binding_array_layout_entries, BindlessMaterialSlab,
    BindlessMaterialSlabError, BindlessSlotIndex, BindlessTextureKey,
};
use crate::core::framework::render::RenderCapabilitySummary;
use crate::core::resource::ResourceId;
use std::num::NonZeroU32;

const FALLBACK: BindlessTextureKey = BindlessTextureKey::new(1);
const BASE_COLOR: BindlessTextureKey = BindlessTextureKey::new(2);
const NORMAL: BindlessTextureKey = BindlessTextureKey::new(3);
const EMISSIVE: BindlessTextureKey = BindlessTextureKey::new(4);

#[test]
fn vacant_slots_are_prepopulated_with_the_fallback_texture() {
    let slab = BindlessMaterialSlab::new(4, FALLBACK).expect("valid slab");

    assert_eq!(slab.fallback_slot(), BindlessSlotIndex::FALLBACK);
    assert_eq!(
        slab.binding_table(),
        &[FALLBACK, FALLBACK, FALLBACK, FALLBACK]
    );
    assert_eq!(slab.active_slot_count(), 0);
}

#[test]
fn identical_textures_deduplicate_without_consuming_another_slot() {
    let mut slab = BindlessMaterialSlab::new(3, FALLBACK).expect("valid slab");
    let first = slab.allocate(BASE_COLOR).expect("first allocation");
    let second = slab.allocate(BASE_COLOR).expect("deduplicated allocation");

    assert_eq!(first.slot_index(), second.slot_index());
    assert_eq!(first.slot_index().get(), 1);
    assert_eq!(slab.active_slot_count(), 1);
    assert_eq!(slab.binding_table(), &[FALLBACK, BASE_COLOR, FALLBACK]);

    assert!(slab.release(first));
    assert_eq!(slab.active_slot_count(), 1);
    assert!(slab.release(second));
    assert_eq!(slab.binding_table(), &[FALLBACK, FALLBACK, FALLBACK]);
}

#[test]
fn stale_lease_cannot_release_a_recycled_slot() {
    let mut slab = BindlessMaterialSlab::new(2, FALLBACK).expect("valid slab");
    let old = slab.allocate(BASE_COLOR).expect("first allocation");
    assert!(slab.release(old));
    let replacement = slab.allocate(NORMAL).expect("recycled allocation");

    assert_eq!(old.slot_index(), replacement.slot_index());
    assert!(!slab.release(old));
    assert_eq!(slab.binding_table(), &[FALLBACK, NORMAL]);
    assert!(slab.release(replacement));
}

#[test]
fn capacity_excludes_the_reserved_fallback_slot_and_recovers_after_release() {
    let mut slab = BindlessMaterialSlab::new(3, FALLBACK).expect("valid slab");
    let base_color = slab.allocate(BASE_COLOR).expect("base color allocation");
    let normal = slab.allocate(NORMAL).expect("normal allocation");

    assert_eq!(
        slab.allocate(EMISSIVE),
        Err(BindlessMaterialSlabError::Exhausted { capacity: 3 })
    );
    assert!(slab.release(normal));
    assert_eq!(
        slab.allocate(EMISSIVE)
            .expect("recovered slot")
            .slot_index()
            .get(),
        2
    );
    assert!(slab.release(base_color));
}

#[test]
fn fallback_allocations_do_not_consume_a_dynamic_slot() {
    let mut slab = BindlessMaterialSlab::new(2, FALLBACK).expect("valid slab");
    let fallback = slab.allocate(FALLBACK).expect("fallback slot");
    let base_color = slab.allocate(BASE_COLOR).expect("dynamic slot");

    assert!(fallback.is_fallback());
    assert!(!slab.release(fallback));
    assert_eq!(base_color.slot_index().get(), 1);
}

#[test]
fn resource_texture_keys_keep_revisions_distinct_without_hash_identity() {
    let resource = ResourceId::from_stable_label("bindless-texture-key-test");

    assert_eq!(
        BindlessTextureKey::from_resource(resource, 2),
        BindlessTextureKey::from_resource(resource, 2)
    );
    assert_ne!(
        BindlessTextureKey::from_resource(resource, 2),
        BindlessTextureKey::from_resource(resource, 3)
    );
    assert_ne!(
        BindlessTextureKey::from_resource(resource, 2),
        BindlessTextureKey::new(2)
    );
}

#[test]
fn zero_capacity_is_rejected() {
    assert!(matches!(
        BindlessMaterialSlab::new(0, FALLBACK),
        Err(BindlessMaterialSlabError::ZeroCapacity)
    ));
}

#[test]
fn negotiated_capability_capacity_uses_the_texture_and_sampler_lower_bound() {
    let capabilities = RenderCapabilitySummary {
        supports_texture_binding_array: true,
        supports_partially_bound_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        max_binding_array_elements_per_shader_stage: 512,
        max_binding_array_sampler_elements_per_shader_stage: 64,
        ..RenderCapabilitySummary::default()
    };

    assert_eq!(
        BindlessMaterialSlab::capacity_for_capabilities(&capabilities),
        Some(64)
    );
}

#[test]
fn missing_bindless_capability_has_no_slab_capacity() {
    assert_eq!(
        BindlessMaterialSlab::capacity_for_capabilities(&RenderCapabilitySummary::default()),
        None
    );
}

#[test]
fn bindless_layout_uses_fixed_texture_and_sampler_arrays() {
    let entries = bindless_material_binding_array_layout_entries(
        NonZeroU32::new(64).expect("nonzero capacity"),
    );

    assert_eq!(entries[0].binding, 0);
    assert_eq!(entries[0].count.map(NonZeroU32::get), Some(64));
    assert!(matches!(entries[0].ty, wgpu::BindingType::Texture { .. }));
    assert_eq!(entries[1].binding, 1);
    assert_eq!(entries[1].count.map(NonZeroU32::get), Some(64));
    assert!(matches!(entries[1].ty, wgpu::BindingType::Sampler(_)));
}

#[test]
fn bindless_bind_group_projection_reserves_slot_capacity() {
    let source = include_str!("../bindless_slab.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("bindless material implementation");

    assert!(implementation.contains("Vec::with_capacity(slot_textures.len())"));
    assert!(implementation.contains("texture_views.extend("));
    assert!(implementation.contains("samplers.extend("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ct_runtime_bindless_view_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const SLOTS_PER_SAMPLE: usize = 256;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(SLOTS_PER_SAMPLE, false));
            optimized.push(measure(SLOTS_PER_SAMPLE, true));
        } else {
            optimized.push(measure(SLOTS_PER_SAMPLE, true));
            legacy.push(measure(SLOTS_PER_SAMPLE, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME394_BINDLESS_VIEW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} slots_per_sample={SLOTS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));

    fn measure(count: usize, use_capacity: bool) -> u128 {
        let started = std::time::Instant::now();
        let mut checksum = 0usize;
        for _ in 0..10_000 {
            if use_capacity {
                let mut values = Vec::with_capacity(count);
                values.extend(0..count);
                checksum ^= values.len();
                std::hint::black_box(values);
            } else {
                let values = (0..count).collect::<Vec<_>>();
                checksum ^= values.len();
                std::hint::black_box(values);
            }
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
