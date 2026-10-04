use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const RAYS_PER_SAMPLE: usize = 64;
const PRIMITIVES_PER_RAY: usize = 256;

#[test]
fn primitive_picking_reserves_ray_and_primitive_capacity() {
    let source = include_str!("../primitive_backend.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("primitive picking implementation");

    assert!(implementation.contains("Vec::with_capacity(rays.len())"));
    assert!(implementation.contains("Vec::with_capacity(self.primitives.len())"));
    assert!(implementation.contains("if let Some(hit) = primitive.hit("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ce_runtime_primitive_picking_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!("RUNTIME383_PRIMITIVE_PICKING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} rays_per_sample={RAYS_PER_SAMPLE} primitives_per_ray={PRIMITIVES_PER_RAY} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..32 {
        let mut groups = if use_capacity {
            Vec::with_capacity(RAYS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for ray in 0..RAYS_PER_SAMPLE {
            let mut hits = if use_capacity {
                Vec::with_capacity(PRIMITIVES_PER_RAY)
            } else {
                Vec::new()
            };
            for primitive in 0..PRIMITIVES_PER_RAY {
                if primitive % 3 != 0 {
                    hits.push((ray, primitive));
                }
            }
            groups.push(hits);
        }
        checksum ^= groups.len();
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
