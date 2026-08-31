use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::{btree_map::Entry, BTreeMap},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use woc_protocol::{
    decode_command_value, encode_command_value, CommandValue, CommandValueLimits, ProtocolError,
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

#[test]
fn command_values_round_trip_nested_data_with_canonical_object_order() {
    let unordered = CommandValue::object(vec![
        (
            "z".to_owned(),
            CommandValue::Array(vec![CommandValue::Number(1.25), CommandValue::Null]),
        ),
        ("a".to_owned(), CommandValue::Bool(true)),
    ])
    .expect("unique object keys");
    let ordered = CommandValue::object(vec![
        ("a".to_owned(), CommandValue::Bool(true)),
        (
            "z".to_owned(),
            CommandValue::Array(vec![CommandValue::Number(1.25), CommandValue::Null]),
        ),
    ])
    .expect("unique object keys");

    let unordered_bytes = encode_command_value(&unordered, CommandValueLimits::default())
        .expect("encode unordered object");
    assert_eq!(
        unordered_bytes,
        encode_command_value(&ordered, CommandValueLimits::default())
            .expect("encode ordered object")
    );
    assert_eq!(
        decode_command_value(&unordered_bytes, CommandValueLimits::default())
            .expect("decode canonical object"),
        ordered
    );
}

#[test]
fn command_value_normalizes_negative_zero_and_rejects_non_finite_numbers() {
    let encoded = encode_command_value(&CommandValue::Number(-0.0), CommandValueLimits::default())
        .expect("negative zero is JSON-equivalent to zero");
    assert_eq!(
        decode_command_value(&encoded, CommandValueLimits::default()).expect("decode zero"),
        CommandValue::Number(0.0)
    );
    assert!(matches!(
        encode_command_value(&CommandValue::Number(f64::NAN), CommandValueLimits::default()),
        Err(ProtocolError::NonFinite {
            field: "command value number",
            value,
        }) if value.is_nan()
    ));
}

#[test]
fn command_value_rejects_duplicate_keys_unknown_tags_and_trailing_bytes() {
    assert_eq!(
        CommandValue::object(vec![
            ("a".to_owned(), CommandValue::Null),
            ("a".to_owned(), CommandValue::Bool(true)),
        ]),
        Err(ProtocolError::DuplicateCommandObjectKey {
            key: "a".to_owned(),
        })
    );

    let duplicate_key = [6, 2, 0, 0, 0, 1, 0, 0, 0, b'a', 0, 1, 0, 0, 0, b'a', 0];
    assert_eq!(
        decode_command_value(&duplicate_key, CommandValueLimits::default()),
        Err(ProtocolError::DuplicateCommandObjectKey {
            key: "a".to_owned(),
        })
    );
    assert_eq!(
        decode_command_value(&[255], CommandValueLimits::default()),
        Err(ProtocolError::UnknownCommandValueTag(255))
    );
    assert_eq!(
        decode_command_value(&[0, 0], CommandValueLimits::default()),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
}

#[test]
fn command_value_limits_reject_deep_or_oversized_inputs() {
    let deeply_nested = CommandValue::Array(vec![CommandValue::Array(vec![CommandValue::Null])]);
    let limits = CommandValueLimits {
        max_value_depth: 1,
        ..CommandValueLimits::default()
    };
    assert_eq!(
        encode_command_value(&deeply_nested, limits),
        Err(ProtocolError::CollectionTooLarge {
            context: "command value depth",
            actual: 2,
            maximum: 1,
        })
    );

    let limits = CommandValueLimits {
        max_total_bytes: 4,
        ..CommandValueLimits::default()
    };
    assert_eq!(
        encode_command_value(&CommandValue::String("wolf".to_owned()), limits),
        Err(ProtocolError::CollectionTooLarge {
            context: "command value bytes",
            actual: 9,
            maximum: 4,
        })
    );
}

const PERFORMANCE_ENTRY_COUNT: usize = 4_096;
const PERFORMANCE_OBJECTS_PER_SAMPLE: usize = 7;
const PERFORMANCE_WARMUPS: usize = 5;
const PERFORMANCE_SAMPLES: usize = 31;

struct ObjectMeasurement {
    elapsed: Duration,
    allocations: usize,
    allocated_bytes: usize,
    checksum: u64,
}

fn performance_fixture() -> Vec<(String, CommandValue)> {
    (0..PERFORMANCE_ENTRY_COUNT)
        .map(|index| {
            (
                format!("command-value-object-key-{index:08}-payload-field"),
                CommandValue::Number((index as u64).wrapping_mul(17) as f64),
            )
        })
        .collect()
}

fn start_counting_allocations() -> Instant {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    COUNTING_ALLOCATIONS.store(true, Ordering::Relaxed);
    Instant::now()
}

fn object_checksum(values: &BTreeMap<String, CommandValue>) -> u64 {
    values.iter().fold(0_u64, |checksum, (key, value)| {
        let value_bits = match value {
            CommandValue::Number(value) => value.to_bits(),
            _ => unreachable!("performance fixture contains only numeric values"),
        };
        checksum
            .wrapping_add(key.len() as u64)
            .wrapping_add(value_bits)
            .rotate_left(5)
    })
}

fn finish_object_measurement(started: Instant, checksum: u64) -> ObjectMeasurement {
    let elapsed = started.elapsed();
    COUNTING_ALLOCATIONS.store(false, Ordering::Relaxed);
    ObjectMeasurement {
        elapsed,
        allocations: ALLOCATION_COUNT.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        checksum: black_box(checksum),
    }
}

#[inline(never)]
fn legacy_clone_then_insert(objects: Vec<Vec<(String, CommandValue)>>) -> ObjectMeasurement {
    let started = start_counting_allocations();
    let mut checksum = 0_u64;
    for entries in black_box(objects) {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            assert!(values.insert(key.clone(), value).is_none());
        }
        checksum = checksum.wrapping_add(object_checksum(black_box(&values)));
    }
    finish_object_measurement(started, black_box(checksum))
}

#[inline(never)]
fn optimized_owned_entry(objects: Vec<Vec<(String, CommandValue)>>) -> ObjectMeasurement {
    let started = start_counting_allocations();
    let mut checksum = 0_u64;
    for entries in black_box(objects) {
        let object = CommandValue::object(entries).expect("performance fixture keys are unique");
        let CommandValue::Object(values) = object else {
            unreachable!("object constructor returns an object")
        };
        checksum = checksum.wrapping_add(object_checksum(black_box(&values)));
    }
    finish_object_measurement(started, black_box(checksum))
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
fn runtime19_batch_command_value_owned_object_keys_release_performance() {
    const MARKER: &str = "RUNTIME19_COMMAND_VALUE_OBJECT_KEYS_BENCH_V1";

    let base = performance_fixture();
    let base_objects = (0..PERFORMANCE_OBJECTS_PER_SAMPLE)
        .map(|_| base.clone())
        .collect::<Vec<_>>();
    for _ in 0..PERFORMANCE_WARMUPS {
        black_box(legacy_clone_then_insert(base_objects.clone()));
        black_box(optimized_owned_entry(base_objects.clone()));
    }

    let mut legacy_ns = Vec::with_capacity(PERFORMANCE_SAMPLES);
    let mut optimized_ns = Vec::with_capacity(PERFORMANCE_SAMPLES);
    let mut reference = None;
    for sample in 0..PERFORMANCE_SAMPLES {
        let legacy_objects = base_objects.clone();
        let optimized_objects = base_objects.clone();
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                legacy_clone_then_insert(legacy_objects),
                optimized_owned_entry(optimized_objects),
            )
        } else {
            let optimized = optimized_owned_entry(optimized_objects);
            let legacy = legacy_clone_then_insert(legacy_objects);
            (legacy, optimized)
        };
        assert_eq!(legacy.checksum, optimized.checksum);
        let sample_reference = (
            legacy.allocations,
            optimized.allocations,
            legacy.allocated_bytes,
            optimized.allocated_bytes,
            legacy.checksum,
        );
        assert_eq!(*reference.get_or_insert(sample_reference), sample_reference);
        legacy_ns.push(legacy.elapsed.as_nanos());
        optimized_ns.push(optimized.elapsed.as_nanos());
    }

    let (legacy_allocations, optimized_allocations, legacy_bytes, optimized_bytes, checksum) =
        reference.expect("at least one performance sample");
    let legacy_p50 = nearest_rank(&mut legacy_ns.clone(), 50);
    let optimized_p50 = nearest_rank(&mut optimized_ns.clone(), 50);
    let legacy_p95 = nearest_rank(&mut legacy_ns, 95);
    let optimized_p95 = nearest_rank(&mut optimized_ns, 95);

    assert!(optimized_allocations.saturating_mul(5) <= legacy_allocations);
    assert!(optimized_bytes.saturating_mul(100) <= legacy_bytes.saturating_mul(60));
    assert!(optimized_p50.saturating_mul(100) <= legacy_p50.saturating_mul(85));
    assert!(optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90));
    println!(
        "{MARKER} entries_per_object={PERFORMANCE_ENTRY_COUNT} \
         objects_per_sample={PERFORMANCE_OBJECTS_PER_SAMPLE} samples={PERFORMANCE_SAMPLES} \
         warmups={PERFORMANCE_WARMUPS} legacy_allocations={legacy_allocations} \
         optimized_allocations={optimized_allocations} allocation_reduction_bps={} \
         legacy_allocated_bytes={legacy_bytes} optimized_allocated_bytes={optimized_bytes} \
         allocated_byte_reduction_bps={} legacy_p50_ns={legacy_p50} \
         optimized_p50_ns={optimized_p50} p50_reduction_bps={} legacy_p95_ns={legacy_p95} \
         optimized_p95_ns={optimized_p95} p95_reduction_bps={} checksum={checksum}",
        reduction_bps(legacy_allocations as u128, optimized_allocations as u128),
        reduction_bps(legacy_bytes as u128, optimized_bytes as u128),
        reduction_bps(legacy_p50, optimized_p50),
        reduction_bps(legacy_p95, optimized_p95),
    );
}
