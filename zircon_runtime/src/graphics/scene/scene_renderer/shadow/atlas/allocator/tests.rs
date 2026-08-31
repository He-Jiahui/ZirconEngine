use super::*;
use std::hint::black_box;
use std::time::Instant;

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
    assert!(
        frame
            .allocations
            .iter()
            .all(|allocation| allocation.allocated_tier == ShadowResolutionTier::T256)
    );
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
fn render_shadow_atlas_preemption_keeps_every_qualifying_priority_pair() {
    let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(512));
    allocator.allocate_frame(
        &(1..=4)
            .map(|light_id| {
                request(
                    light_id,
                    ShadowResolutionTier::T256,
                    ShadowResolutionTier::T256,
                    light_id as f32,
                )
            })
            .collect::<Vec<_>>(),
    );

    allocator.allocate_frame(
        &(1..=5)
            .map(|light_id| {
                request(
                    light_id,
                    ShadowResolutionTier::T256,
                    ShadowResolutionTier::T256,
                    light_id as f32,
                )
            })
            .collect::<Vec<_>>(),
    );

    let expected = [
        (key(2), key(1)),
        (key(3), key(1)),
        (key(3), key(2)),
        (key(4), key(1)),
        (key(4), key(2)),
        (key(4), key(3)),
        (key(5), key(1)),
        (key(5), key(2)),
        (key(5), key(3)),
        (key(5), key(4)),
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    let actual = allocator.preemption.keys().copied().collect::<HashSet<_>>();

    assert_eq!(actual, expected);
}

#[test]
#[ignore = "release-only shadow preemption contention benchmark"]
fn shadow_preemption_index_release_benchmark_evidence() {
    const SLOT_COUNT: usize = 4_096;
    const SAMPLE_PAIRS: usize = 21;

    fn legacy_contention(
        config: ShadowAtlasConfig,
        previous: &HashMap<ShadowSlotKey, RetainedShadowSlot>,
        planned: &[PlannedShadowSlot],
    ) -> usize {
        let mut active_pairs = HashSet::new();
        for retained_key in previous.keys().copied() {
            let Some(incumbent) = planned
                .iter()
                .find(|slot| slot.request.key == retained_key)
                .copied()
            else {
                continue;
            };
            let required_priority =
                incumbent.request.priority_score() * config.preemption_score_multiplier.max(1.0);
            for challenger in planned.iter().copied() {
                if challenger.request.key == incumbent.request.key {
                    continue;
                }
                if challenger.request.priority_score() < required_priority {
                    continue;
                }
                if challenger.allocated_tier.size_px() < incumbent.allocated_tier.size_px() {
                    continue;
                }
                active_pairs.insert((challenger.request.key, incumbent.request.key));
            }
        }
        active_pairs.len()
    }

    fn measure(mut operation: impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        assert_eq!(black_box(operation()), 0);
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let config = ShadowAtlasConfig::new_square(4_096);
    let mut planned = (0..SLOT_COUNT)
        .map(|index| PlannedShadowSlot {
            request: request(
                index as u64,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T256,
                1.0,
            ),
            allocated_tier: ShadowResolutionTier::T256,
        })
        .collect::<Vec<_>>();
    planned.sort_by(compare_planned_slots);
    let planned_by_key = planned
        .iter()
        .map(|slot| (slot.request.key, *slot))
        .collect::<HashMap<_, _>>();
    let previous = planned
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            (
                slot.request.key,
                RetainedShadowSlot {
                    allocation: ShadowSlotAllocation {
                        key: slot.request.key,
                        rect: ShadowAtlasRect::new(index as u32, 0, 1, 1),
                        requested_tier: slot.request.requested_tier,
                        allocated_tier: slot.allocated_tier,
                        priority: slot.request.priority_score(),
                        reused_previous: false,
                    },
                    last_seen_frame: 0,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let mut allocator = ShadowAtlasAllocator::new(config);
    allocator.previous = previous;

    assert_eq!(legacy_contention(config, &allocator.previous, &planned), 0);
    allocator.update_preemption_contention(&planned, &planned_by_key, true);
    assert!(allocator.preemption.is_empty());

    for _ in 0..3 {
        black_box(legacy_contention(
            config,
            black_box(&allocator.previous),
            black_box(&planned),
        ));
        allocator.update_preemption_contention(
            black_box(&planned),
            black_box(&planned_by_key),
            true,
        );
        black_box(allocator.preemption.len());
    }

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure(|| {
                legacy_contention(config, &allocator.previous, &planned)
            }));
            indexed_ns.push(measure(|| {
                allocator.update_preemption_contention(&planned, &planned_by_key, true);
                allocator.preemption.len()
            }));
        } else {
            indexed_ns.push(measure(|| {
                allocator.update_preemption_contention(&planned, &planned_by_key, true);
                allocator.preemption.len()
            }));
            legacy_ns.push(measure(|| {
                legacy_contention(config, &allocator.previous, &planned)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let indexed_p50_ns = nearest_rank(&indexed_ns, 50);
    let indexed_p95_ns = nearest_rank(&indexed_ns, 95);
    let legacy_incumbent_linear_comparisons = SLOT_COUNT * (SLOT_COUNT + 1) / 2;
    let legacy_challenger_visits = SLOT_COUNT * SLOT_COUNT;
    let indexed_hash_probes = SLOT_COUNT;
    let indexed_challenger_visits = SLOT_COUNT;

    println!(
        "RUNTIME09E_SHADOW_PREEMPTION_BENCH_V1 slots={SLOT_COUNT} sample_pairs={SAMPLE_PAIRS} workload=oversubscribed_equal_priority legacy_incumbent_linear_comparisons={legacy_incumbent_linear_comparisons} legacy_challenger_visits={legacy_challenger_visits} indexed_hash_probes={indexed_hash_probes} indexed_challenger_visits={indexed_challenger_visits} legacy_p50_ns={legacy_p50_ns} indexed_p50_ns={indexed_p50_ns} legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} legacy_raw_ns={} indexed_raw_ns={}",
        raw(&legacy_ns),
        raw(&indexed_ns),
    );

    assert!(
        indexed_p95_ns.saturating_mul(20) <= legacy_p95_ns,
        "indexed preemption lookup must reduce P95 by at least 95%: legacy={legacy_p95_ns}ns indexed={indexed_p95_ns}ns"
    );
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
