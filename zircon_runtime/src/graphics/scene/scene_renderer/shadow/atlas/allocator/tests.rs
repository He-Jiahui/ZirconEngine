use super::*;

fn key(light_id: u64) -> ShadowSlotKey {
    ShadowSlotKey::new(light_id, 0)
}

fn request(
    light_id: u64,
    tier: ShadowResolutionTier,
    minimum_tier: ShadowResolutionTier,
    priority: f32,
) -> ShadowSlotRequest {
    ShadowSlotRequest::new(key(light_id), tier, priority).with_minimum_tier(minimum_tier)
}

fn no_allocations_overlap(allocations: &[ShadowSlotAllocation]) -> bool {
    for (index, lhs) in allocations.iter().enumerate() {
        for rhs in allocations.iter().skip(index + 1) {
            if lhs.rect.intersects(rhs.rect) {
                return false;
            }
        }
    }
    true
}

#[test]
fn render_shadow_atlas_allocates_tiers_descending() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(512));
    let frame = allocator.allocate_frame(&[
        request(
            1,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T128,
            1.0,
        ),
        request(
            2,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T128,
            1.0,
        ),
        request(
            3,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T128,
            1.0,
        ),
        request(
            4,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T128,
            1.0,
        ),
    ]);

    assert_eq!(frame.scale_factor, 1);
    assert_eq!(frame.allocations.len(), 4);
    assert!(frame.rejected.is_empty());
    assert!(frame
        .allocations
        .iter()
        .all(|allocation| allocation.allocated_tier == ShadowResolutionTier::T256));
    assert!(no_allocations_overlap(&frame.allocations));
}

#[test]
fn render_shadow_atlas_global_downgrade_fits_pressure() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(512));
    let requests = (0..5)
        .map(|index| {
            request(
                index,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T128,
                1.0,
            )
        })
        .collect::<Vec<_>>();
    let frame = allocator.allocate_frame(&requests);

    assert_eq!(frame.scale_factor, 2);
    assert_eq!(frame.allocations.len(), 5);
    assert!(frame.rejected.is_empty());
    assert!(frame.allocations.iter().all(|allocation| {
        allocation.allocated_tier == ShadowResolutionTier::T128 && allocation.was_downgraded()
    }));
    assert!(no_allocations_overlap(&frame.allocations));
}

#[test]
fn render_shadow_atlas_evicts_lowest_priority_on_pressure() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
    let frame = allocator.allocate_frame(&[
        request(
            1,
            ShadowResolutionTier::T128,
            ShadowResolutionTier::T128,
            1.0,
        ),
        request(
            2,
            ShadowResolutionTier::T128,
            ShadowResolutionTier::T128,
            2.0,
        ),
        request(
            3,
            ShadowResolutionTier::T128,
            ShadowResolutionTier::T128,
            3.0,
        ),
        request(
            4,
            ShadowResolutionTier::T128,
            ShadowResolutionTier::T128,
            4.0,
        ),
        request(
            5,
            ShadowResolutionTier::T128,
            ShadowResolutionTier::T128,
            5.0,
        ),
    ]);

    assert_eq!(frame.allocations.len(), 4);
    assert_eq!(frame.rejected.len(), 1);
    assert!(frame.allocation_for(key(5)).is_some());
    assert!(frame.allocation_for(key(4)).is_some());
    assert!(frame.allocation_for(key(3)).is_some());
    assert!(frame.allocation_for(key(2)).is_some());
    assert_eq!(frame.rejected[0].key, key(1));
}

#[test]
fn render_shadow_atlas_hysteresis_prevents_flapping() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
    let first = allocator.allocate_frame(&[request(
        1,
        ShadowResolutionTier::T256,
        ShadowResolutionTier::T256,
        1.0,
    )]);
    let previous_rect = first.allocation_for(key(1)).unwrap().rect;

    let second = allocator.allocate_frame(&[
        request(
            1,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T256,
            1.0,
        ),
        request(
            2,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T256,
            1.2,
        ),
    ]);

    let retained = second.allocation_for(key(1)).unwrap();
    assert_eq!(retained.rect, previous_rect);
    assert!(retained.reused_previous);
    assert!(second.allocation_for(key(2)).is_none());
}

#[test]
fn render_shadow_atlas_slot_generation_survives_reuse_and_changes_after_reallocation() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
    let first = allocator.allocate_frame(&[request(
        1,
        ShadowResolutionTier::T256,
        ShadowResolutionTier::T256,
        1.0,
    )]);
    let first_generation = first
        .slot_generation_for(key(1))
        .expect("allocated slot has a generation");

    let reused = allocator.allocate_frame(&[request(
        1,
        ShadowResolutionTier::T256,
        ShadowResolutionTier::T256,
        1.0,
    )]);
    assert_eq!(reused.slot_generation_for(key(1)), Some(first_generation));

    allocator.allocate_frame(&[]);
    let reallocated = allocator.allocate_frame(&[request(
        1,
        ShadowResolutionTier::T256,
        ShadowResolutionTier::T256,
        1.0,
    )]);

    assert_ne!(
        reallocated.slot_generation_for(key(1)),
        Some(first_generation)
    );
}

#[test]
fn render_shadow_atlas_preempts_after_confirmed_priority_margin() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
    allocator.allocate_frame(&[request(
        1,
        ShadowResolutionTier::T256,
        ShadowResolutionTier::T256,
        1.0,
    )]);

    for _ in 0..SHADOW_ATLAS_PREEMPTION_FRAMES {
        allocator.allocate_frame(&[
            request(
                1,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T256,
                1.0,
            ),
            request(
                2,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T256,
                1.5,
            ),
        ]);
    }
    let frame = allocator.last_frame();

    assert!(frame.allocation_for(key(1)).is_none());
    assert!(frame.allocation_for(key(2)).is_some());
}

#[test]
fn render_shadow_atlas_scale_bias_matches_slice_transform() {
    let allocation = ShadowSlotAllocation {
        key: key(7),
        rect: ShadowAtlasRect::new(128, 256, 512, 512),
        requested_tier: ShadowResolutionTier::T512,
        allocated_tier: ShadowResolutionTier::T512,
        priority: 1.0,
        reused_previous: false,
    };

    assert_eq!(
        allocation.atlas_scale_bias(2048, 2048),
        [0.25, 0.25, 0.0625, 0.125]
    );
}
