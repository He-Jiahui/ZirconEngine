use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::BTreeMap,
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use woc_protocol::{
    EntityRef, MovementFrame, MovementFrameBatch, MovementFrameDisposition, MovementInputError,
    MovementInputFlags, MovementInputRelay, MOVEMENT_INPUT_STALE_AFTER_TICKS,
};

struct CountingAllocator;

static COUNTING_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if COUNTING_ALLOCATIONS.load(Ordering::Relaxed) && !pointer.is_null() {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn actor(id: u64, generation: u32) -> EntityRef {
    EntityRef { id, generation }
}

fn flags(forward: bool, back: bool) -> MovementInputFlags {
    MovementInputFlags {
        forward,
        back,
        ..MovementInputFlags::default()
    }
}

fn frame(
    actor: EntityRef,
    sequence: u32,
    flags: MovementInputFlags,
    facing: Option<f64>,
) -> MovementFrame {
    MovementFrame {
        actor,
        sequence,
        flags,
        facing,
    }
}

#[test]
fn movement_batch_canonicalizes_actor_order_and_rejects_invalid_frames() {
    let batch = MovementFrameBatch::new(vec![
        frame(actor(9, 1), 4, flags(true, false), None),
        frame(actor(3, 2), 7, flags(false, true), Some(0.25)),
    ])
    .expect("valid frames must form a batch");
    assert_eq!(
        batch
            .frames()
            .iter()
            .map(|frame| frame.actor)
            .collect::<Vec<_>>(),
        vec![actor(3, 2), actor(9, 1)]
    );

    assert!(matches!(
        MovementFrameBatch::new(vec![frame(actor(0, 1), 1, flags(false, false), None)]),
        Err(MovementInputError::InvalidActor { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![frame(actor(1, 1), 0, flags(false, false), None)]),
        Err(MovementInputError::InvalidSequence { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![frame(
            actor(1, 1),
            1,
            flags(false, false),
            Some(f64::NAN)
        )]),
        Err(MovementInputError::InvalidFacing { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![
            frame(actor(1, 1), 1, flags(false, false), None),
            frame(actor(1, 1), 2, flags(false, false), None),
        ]),
        Err(MovementInputError::DuplicateActor { .. })
    ));
}

#[test]
fn relay_classifies_sequences_without_replacing_newer_input() {
    let player = actor(41, 3);
    let mut relay = MovementInputRelay::default();
    let initial = MovementFrameBatch::new(vec![frame(player, 4, flags(true, false), Some(0.75))])
        .expect("initial frame is valid");
    assert_eq!(
        relay
            .apply_batch(10, &initial)
            .expect("initial batch applies"),
        vec![MovementFrameDisposition::Applied {
            actor: player,
            acknowledgement: 4,
        }]
    );

    let older_sequence =
        MovementFrameBatch::new(vec![frame(player, 3, flags(false, true), Some(0.25))])
            .expect("older-sequence frame shape is still valid");
    assert_eq!(
        relay
            .apply_batch(11, &older_sequence)
            .expect("older-sequence batch is classified"),
        vec![MovementFrameDisposition::Stale {
            actor: player,
            acknowledgement: 4,
        }]
    );
    let after_older_sequence = relay.input(player).expect("player input is retained");
    assert_eq!(after_older_sequence.flags, flags(true, false));
    assert_eq!(after_older_sequence.facing, Some(0.75));
    assert_eq!(after_older_sequence.accepted_tick, 10);
    assert_eq!(relay.acknowledgement(player), Some(4));

    let duplicate = MovementFrameBatch::new(vec![frame(player, 4, flags(false, true), Some(0.5))])
        .expect("duplicate frame shape is still valid");
    assert_eq!(
        relay
            .apply_batch(12, &duplicate)
            .expect("duplicate batch is classified"),
        vec![MovementFrameDisposition::Duplicate {
            actor: player,
            acknowledgement: 4,
        }]
    );
    let after_duplicate = relay.input(player).expect("player input is retained");
    assert_eq!(after_duplicate.flags, flags(true, false));
    assert_eq!(after_duplicate.facing, Some(0.75));
    assert_eq!(after_duplicate.accepted_tick, 10);

    let newer = MovementFrameBatch::new(vec![frame(player, 5, flags(false, true), None)])
        .expect("newer frame is valid");
    assert_eq!(
        relay.apply_batch(13, &newer).expect("newer batch applies"),
        vec![MovementFrameDisposition::Applied {
            actor: player,
            acknowledgement: 5,
        }]
    );
    let active = relay.input(player).expect("player input is retained");
    assert_eq!(active.flags, flags(false, true));
    assert_eq!(active.facing, Some(0.75));
    assert_eq!(active.accepted_tick, 13);
    assert_eq!(relay.acknowledgement(player), Some(5));
}

#[test]
fn relay_clears_held_flags_only_after_the_source_stale_window() {
    let player = actor(7, 1);
    let mut relay = MovementInputRelay::default();
    let batch = MovementFrameBatch::new(vec![frame(player, 1, flags(true, false), Some(-0.5))])
        .expect("frame is valid");
    relay.apply_batch(20, &batch).expect("batch applies");

    assert!(relay
        .clear_stale(20 + MOVEMENT_INPUT_STALE_AFTER_TICKS)
        .expect("exact stale threshold is valid")
        .is_empty());
    assert_eq!(
        relay.input(player).expect("input remains").flags,
        flags(true, false)
    );

    assert_eq!(
        relay
            .clear_stale(21 + MOVEMENT_INPUT_STALE_AFTER_TICKS)
            .expect("post-threshold tick is valid"),
        vec![player]
    );
    let cleared = relay
        .input(player)
        .expect("input state remains for acknowledgement");
    assert_eq!(cleared.flags, MovementInputFlags::default());
    assert_eq!(cleared.facing, Some(-0.5));
    assert_eq!(cleared.acknowledgement, 1);
}

#[test]
fn relay_rejects_tick_regression_before_mutating_input_state() {
    let player = actor(19, 1);
    let mut relay = MovementInputRelay::default();
    let first = MovementFrameBatch::new(vec![frame(player, 1, flags(true, false), None)])
        .expect("frame is valid");
    relay.apply_batch(40, &first).expect("first tick applies");

    let later = MovementFrameBatch::new(vec![frame(player, 2, flags(false, true), None)])
        .expect("frame is valid");
    assert!(matches!(
        relay.apply_batch(39, &later),
        Err(MovementInputError::TickRegression {
            previous: 40,
            actual: 39,
        })
    ));
    let retained = relay.input(player).expect("original input remains");
    assert_eq!(retained.acknowledgement, 1);
    assert_eq!(retained.flags, flags(true, false));
}

const PERFORMANCE_ACTOR_COUNT: usize = 8_192;
const PERFORMANCE_WARMUPS: usize = 5;
const PERFORMANCE_SAMPLES: usize = 31;

#[derive(Clone, Copy)]
struct LegacyInput {
    acknowledgement: u32,
    flags: MovementInputFlags,
    facing: Option<f64>,
    accepted_tick: u64,
}

struct MovementMeasurement {
    elapsed: Duration,
    allocations: usize,
    allocated_bytes: usize,
    logical_probes: usize,
    checksum: u64,
}

fn performance_frames(sequence: u32, alternate_facing: bool) -> Vec<MovementFrame> {
    (1..=PERFORMANCE_ACTOR_COUNT as u64)
        .map(|actor_id| {
            frame(
                actor(actor_id, 1),
                sequence,
                flags(sequence % 2 == 0, sequence % 2 != 0),
                if alternate_facing && actor_id % 2 == 0 {
                    Some(0.5)
                } else if alternate_facing {
                    None
                } else {
                    Some(actor_id as f64 * 0.001)
                },
            )
        })
        .collect()
}

fn legacy_fixture() -> (BTreeMap<(u64, u32), LegacyInput>, Vec<MovementFrame>) {
    let inputs = performance_frames(100, false)
        .into_iter()
        .map(|frame| {
            (
                (frame.actor.id, frame.actor.generation),
                LegacyInput {
                    acknowledgement: frame.sequence,
                    flags: frame.flags,
                    facing: frame.facing,
                    accepted_tick: 50,
                },
            )
        })
        .collect();
    (inputs, performance_frames(101, true))
}

fn optimized_fixture() -> (MovementInputRelay, MovementFrameBatch) {
    let initial = MovementFrameBatch::from_canonical(performance_frames(100, false))
        .expect("initial performance frames are canonical");
    let mut relay = MovementInputRelay::default();
    relay
        .apply_batch(50, &initial)
        .expect("initial performance batch applies");
    let newer = MovementFrameBatch::from_canonical(performance_frames(101, true))
        .expect("newer performance frames are canonical");
    (relay, newer)
}

fn start_counting_allocations() -> Instant {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    COUNTING_ALLOCATIONS.store(true, Ordering::Relaxed);
    Instant::now()
}

fn flags_bits(flags: MovementInputFlags) -> u64 {
    u64::from(flags.forward)
        | (u64::from(flags.back) << 1)
        | (u64::from(flags.turn_left) << 2)
        | (u64::from(flags.turn_right) << 3)
        | (u64::from(flags.strafe_left) << 4)
        | (u64::from(flags.strafe_right) << 5)
        | (u64::from(flags.jump) << 6)
}

fn legacy_checksum(inputs: &BTreeMap<(u64, u32), LegacyInput>, acknowledgements: &[u32]) -> u64 {
    inputs.iter().zip(acknowledgements).fold(
        0_u64,
        |checksum, ((actor, input), acknowledgement)| {
            checksum
                .wrapping_add(actor.0)
                .wrapping_add(u64::from(actor.1))
                .wrapping_add(u64::from(input.acknowledgement))
                .wrapping_add(flags_bits(input.flags))
                .wrapping_add(input.facing.unwrap_or_default().to_bits())
                .wrapping_add(input.accepted_tick)
                .wrapping_add(u64::from(*acknowledgement))
        },
    )
}

fn optimized_checksum(
    relay: &MovementInputRelay,
    dispositions: &[MovementFrameDisposition],
) -> u64 {
    dispositions.iter().fold(0_u64, |checksum, disposition| {
        let (actor, acknowledgement) = match *disposition {
            MovementFrameDisposition::Applied {
                actor,
                acknowledgement,
            }
            | MovementFrameDisposition::Duplicate {
                actor,
                acknowledgement,
            }
            | MovementFrameDisposition::Stale {
                actor,
                acknowledgement,
            } => (actor, acknowledgement),
        };
        let input = relay
            .input(actor)
            .expect("performance actor remains retained");
        checksum
            .wrapping_add(actor.id)
            .wrapping_add(u64::from(actor.generation))
            .wrapping_add(u64::from(input.acknowledgement))
            .wrapping_add(flags_bits(input.flags))
            .wrapping_add(input.facing.unwrap_or_default().to_bits())
            .wrapping_add(input.accepted_tick)
            .wrapping_add(u64::from(acknowledgement))
    })
}

#[inline(never)]
fn legacy_repeated_lookup(
    mut inputs: BTreeMap<(u64, u32), LegacyInput>,
    frames: &[MovementFrame],
) -> MovementMeasurement {
    let started = start_counting_allocations();
    let mut acknowledgements = Vec::with_capacity(frames.len());
    for frame in black_box(frames) {
        let key = (frame.actor.id, frame.actor.generation);
        let acknowledgement = inputs
            .get(&key)
            .map(|previous| previous.acknowledgement.max(frame.sequence))
            .unwrap_or(frame.sequence);
        let facing = frame
            .facing
            .or_else(|| inputs.get(&key).and_then(|previous| previous.facing));
        inputs.insert(
            key,
            LegacyInput {
                acknowledgement,
                flags: frame.flags,
                facing,
                accepted_tick: 51,
            },
        );
        acknowledgements.push(acknowledgement);
    }
    let elapsed = started.elapsed();
    COUNTING_ALLOCATIONS.store(false, Ordering::Relaxed);
    MovementMeasurement {
        elapsed,
        allocations: ALLOCATION_COUNT.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        logical_probes: frames.len().saturating_mul(3),
        checksum: black_box(legacy_checksum(&inputs, &acknowledgements)),
    }
}

#[inline(never)]
fn optimized_entry_lookup(
    mut relay: MovementInputRelay,
    batch: &MovementFrameBatch,
) -> MovementMeasurement {
    let started = start_counting_allocations();
    let dispositions = relay
        .apply_batch(51, black_box(batch))
        .expect("newer performance batch applies");
    let elapsed = started.elapsed();
    COUNTING_ALLOCATIONS.store(false, Ordering::Relaxed);
    MovementMeasurement {
        elapsed,
        allocations: ALLOCATION_COUNT.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        logical_probes: batch.frames().len(),
        checksum: black_box(optimized_checksum(&relay, &dispositions)),
    }
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn reduction_bps(baseline: u128, optimized: u128) -> u128 {
    baseline.saturating_sub(optimized).saturating_mul(10_000) / baseline.max(1)
}

#[test]
#[ignore = "release-only performance evidence"]
fn runtime19_batch_movement_sequence_admission_release_performance() {
    const MARKER: &str = "RUNTIME19_MOVEMENT_SEQUENCE_ADMISSION_BENCH_V1";

    let (base_inputs, frames) = legacy_fixture();
    for _ in 0..PERFORMANCE_WARMUPS {
        black_box(legacy_repeated_lookup(base_inputs.clone(), &frames));
        let (relay, batch) = optimized_fixture();
        black_box(optimized_entry_lookup(relay, &batch));
    }

    let mut legacy_ns = Vec::with_capacity(PERFORMANCE_SAMPLES);
    let mut optimized_ns = Vec::with_capacity(PERFORMANCE_SAMPLES);
    let mut reference = None;
    for sample in 0..PERFORMANCE_SAMPLES {
        let legacy_inputs = base_inputs.clone();
        let (relay, batch) = optimized_fixture();
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                legacy_repeated_lookup(legacy_inputs, &frames),
                optimized_entry_lookup(relay, &batch),
            )
        } else {
            let optimized = optimized_entry_lookup(relay, &batch);
            let legacy = legacy_repeated_lookup(legacy_inputs, &frames);
            (legacy, optimized)
        };
        assert_eq!(legacy.checksum, optimized.checksum);
        let sample_reference = (
            legacy.allocations,
            optimized.allocations,
            legacy.allocated_bytes,
            optimized.allocated_bytes,
            legacy.logical_probes,
            optimized.logical_probes,
            legacy.checksum,
        );
        assert_eq!(*reference.get_or_insert(sample_reference), sample_reference);
        legacy_ns.push(legacy.elapsed.as_nanos());
        optimized_ns.push(optimized.elapsed.as_nanos());
    }

    let (
        legacy_allocations,
        optimized_allocations,
        legacy_bytes,
        optimized_bytes,
        legacy_probes,
        optimized_probes,
        checksum,
    ) = reference.expect("at least one performance sample");
    let legacy_p50 = nearest_rank(&mut legacy_ns.clone(), 50);
    let optimized_p50 = nearest_rank(&mut optimized_ns.clone(), 50);
    let legacy_p95 = nearest_rank(&mut legacy_ns, 95);
    let optimized_p95 = nearest_rank(&mut optimized_ns, 95);

    assert!(optimized_probes.saturating_mul(5) <= legacy_probes.saturating_mul(2));
    assert!(optimized_p50.saturating_mul(100) <= legacy_p50.saturating_mul(60));
    assert!(optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(60));
    println!(
        "{MARKER} actors={PERFORMANCE_ACTOR_COUNT} samples={PERFORMANCE_SAMPLES} \
         warmups={PERFORMANCE_WARMUPS} legacy_tree_probes={legacy_probes} \
         optimized_tree_probes={optimized_probes} probe_reduction_bps={} \
         legacy_allocations={legacy_allocations} optimized_allocations={optimized_allocations} \
         legacy_allocated_bytes={legacy_bytes} optimized_allocated_bytes={optimized_bytes} \
         legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} p50_reduction_bps={} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} p95_reduction_bps={} \
         checksum={checksum}",
        reduction_bps(legacy_probes as u128, optimized_probes as u128),
        reduction_bps(legacy_p50, optimized_p50),
        reduction_bps(legacy_p95, optimized_p95),
    );
}
