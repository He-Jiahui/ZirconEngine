use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const CUBEMAP_COUNT: usize = 4_096;
const FACE_COUNT: usize = 6;
const WARMUPS: usize = 3;
const SAMPLES: usize = 21;

struct CountingAllocator;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if COUNTING.load(Ordering::Relaxed) && !pointer.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
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

#[derive(Clone)]
struct Descriptor {
    format: String,
    usage: Vec<u32>,
    asset_usage: Vec<u32>,
    metadata: Vec<u8>,
    mip_count: u32,
}

#[derive(Clone)]
struct Face {
    descriptor: Option<Descriptor>,
}

struct Measurement {
    elapsed: Duration,
    allocations: usize,
    allocated_bytes: usize,
    checksum: u64,
}

fn fixture() -> Vec<Vec<Face>> {
    (0..CUBEMAP_COUNT)
        .map(|cubemap| {
            (0..FACE_COUNT)
                .map(|face| Face {
                    descriptor: Some(Descriptor {
                        format: format!("rgba16float-cubemap-{cubemap:08}-face-{face}"),
                        usage: (0..8).collect(),
                        asset_usage: (0..4).collect(),
                        metadata: vec![((cubemap + face) & 0xff) as u8; 64],
                        mip_count: 5,
                    }),
                })
                .collect()
        })
        .collect()
}

fn descriptor_checksum(descriptor: &Descriptor) -> u64 {
    descriptor.format.len() as u64
        + descriptor
            .usage
            .iter()
            .map(|value| u64::from(*value))
            .sum::<u64>()
        + descriptor
            .asset_usage
            .iter()
            .map(|value| u64::from(*value))
            .sum::<u64>()
        + descriptor
            .metadata
            .iter()
            .map(|value| u64::from(*value))
            .sum::<u64>()
        + u64::from(descriptor.mip_count)
}

fn start_counting() -> Instant {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    Instant::now()
}

fn finish_counting(started: Instant, checksum: u64) -> Measurement {
    let elapsed = started.elapsed();
    COUNTING.store(false, Ordering::Relaxed);
    Measurement {
        elapsed,
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        checksum,
    }
}

fn legacy(cubemaps: Vec<Vec<Face>>) -> Measurement {
    let started = start_counting();
    let mut checksum = 0_u64;
    for faces in black_box(&cubemaps) {
        let first_descriptor = black_box(faces[0].descriptor.as_ref().unwrap().clone());
        checksum = checksum.wrapping_add(descriptor_checksum(&first_descriptor));
        for face in faces {
            let descriptor = black_box(face.descriptor.as_ref().unwrap().clone());
            checksum = checksum.wrapping_add(descriptor_checksum(&descriptor));
        }
    }
    finish_counting(started, black_box(checksum))
}

fn optimized(mut cubemaps: Vec<Vec<Face>>) -> Measurement {
    let started = start_counting();
    let mut checksum = 0_u64;
    for faces in black_box(&mut cubemaps) {
        let first_descriptor = black_box(faces[0].descriptor.take().unwrap());
        checksum = checksum
            .wrapping_add(descriptor_checksum(&first_descriptor))
            .wrapping_add(descriptor_checksum(&first_descriptor));
        for face in faces.iter_mut().skip(1) {
            let descriptor = black_box(face.descriptor.take().unwrap());
            checksum = checksum.wrapping_add(descriptor_checksum(&descriptor));
        }
    }
    finish_counting(started, black_box(checksum))
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * percentile / 100]
}

fn reduction(baseline: u128, optimized: u128) -> f64 {
    (baseline as f64 - optimized as f64) * 100.0 / baseline as f64
}

fn main() {
    let base = fixture();
    for _ in 0..WARMUPS {
        black_box(legacy(base.clone()));
        black_box(optimized(base.clone()));
    }

    let mut legacy_ns = Vec::with_capacity(SAMPLES);
    let mut optimized_ns = Vec::with_capacity(SAMPLES);
    let mut reference = None;
    for sample in 0..SAMPLES {
        let legacy_input = base.clone();
        let optimized_input = base.clone();
        let (legacy_measurement, optimized_measurement) = if sample % 2 == 0 {
            (legacy(legacy_input), optimized(optimized_input))
        } else {
            let optimized_measurement = optimized(optimized_input);
            let legacy_measurement = legacy(legacy_input);
            (legacy_measurement, optimized_measurement)
        };
        assert_eq!(legacy_measurement.checksum, optimized_measurement.checksum);
        reference.get_or_insert((
            legacy_measurement.allocations,
            optimized_measurement.allocations,
            legacy_measurement.allocated_bytes,
            optimized_measurement.allocated_bytes,
            legacy_measurement.checksum,
        ));
        legacy_ns.push(legacy_measurement.elapsed.as_nanos());
        optimized_ns.push(optimized_measurement.elapsed.as_nanos());
    }

    let (legacy_allocations, optimized_allocations, legacy_bytes, optimized_bytes, checksum) =
        reference.expect("at least one sample");
    let legacy_p50 = percentile(&mut legacy_ns.clone(), 50);
    let optimized_p50 = percentile(&mut optimized_ns.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_ns, 95);
    let optimized_p95 = percentile(&mut optimized_ns, 95);
    assert_eq!(legacy_allocations, CUBEMAP_COUNT * FACE_COUNT * 4 * 7 / 6);
    assert_eq!(optimized_allocations, 0);
    assert_eq!(optimized_bytes, 0);
    assert!(optimized_p50 * 100 <= legacy_p50 * 65);
    assert!(optimized_p95 * 100 <= legacy_p95 * 65);
    println!("RUNTIME92_OWNED_CUBEMAP_FACE_DESCRIPTORS_MODEL_V1 cubemaps={CUBEMAP_COUNT} faces={FACE_COUNT} samples={SAMPLES} warmups={WARMUPS} allocations_legacy={legacy_allocations} allocations_optimized={optimized_allocations} allocated_bytes_legacy={legacy_bytes} allocated_bytes_optimized={optimized_bytes} p50_ns_legacy={legacy_p50} p50_ns_optimized={optimized_p50} p95_ns_legacy={legacy_p95} p95_ns_optimized={optimized_p95} p50_reduction_percent={:.3} p95_reduction_percent={:.3} checksum={checksum}", reduction(legacy_p50, optimized_p50), reduction(legacy_p95, optimized_p95));
}
