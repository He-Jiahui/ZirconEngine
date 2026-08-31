use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::{Transform, Vec3};
use crate::scene::components::ColliderShape;

use super::visit_collider_shape_property_entries;

const SAMPLE_PAIRS: usize = 31;
const TRAVERSALS_PER_SAMPLE: usize = 5_000;
const POINT_COUNT: usize = 64;

#[test]
fn optimization_batch_20260829iv_runtime294_reused_path_buffer_preserves_recursive_paths() {
    let shape = ColliderShape::Compound {
        children: vec![(
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            Box::new(ColliderShape::Sphere { radius: 0.5 }),
        )],
    };
    let mut paths = Vec::new();

    assert!(visit_collider_shape_property_entries(
        &shape,
        "Collider.shape",
        &mut |path, value, _| {
            paths.push(path.to_string());
            black_box(value());
            true
        },
    ));

    assert_eq!(
        paths.iter().map(String::as_str).collect::<Vec<_>>(),
        [
            "Collider.shape.kind",
            "Collider.shape.child_count",
            "Collider.shape.children.0.transform.translation",
            "Collider.shape.children.0.transform.rotation",
            "Collider.shape.children.0.transform.scale",
            "Collider.shape.children.0.shape.kind",
            "Collider.shape.children.0.shape.radius",
        ]
    );
}

#[test]
fn optimization_batch_20260829iv_runtime294_property_walk_reuses_one_path_string() {
    let source = include_str!("../collider_shape.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("String::with_capacity"));
    assert!(implementation.contains("visit_collider_shape_property_entries_with_path"));
    assert!(!implementation.contains("let path = format!(\"{prefix}.{}\", $suffix);"));
    assert!(!implementation.contains("let child_prefix = format!"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829iv_runtime294_reused_collider_property_path_buffer_bench() {
    let shape = ColliderShape::ConvexHull {
        points: (0..POINT_COUNT)
            .map(|index| Vec3::new(index as f32, 1.0, 2.0))
            .collect(),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&shape, false));
            optimized_samples.push(measure(&shape, true));
        } else {
            optimized_samples.push(measure(&shape, true));
            legacy_samples.push(measure(&shape, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME294_REUSED_COLLIDER_PROPERTY_PATH_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
traversals_per_sample={TRAVERSALS_PER_SAMPLE} point_count={POINT_COUNT} \
legacy_path_allocations_per_traversal=66 optimized_path_allocations_per_traversal=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_visit_convex_hull<F>(shape: &ColliderShape, prefix: &str, visitor: &mut F) -> bool
where
    F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
{
    let ColliderShape::ConvexHull { points } = shape else {
        return false;
    };
    let path = format!("{prefix}.kind");
    let mut build_kind = || ScenePropertyValue::Enum("convex_hull".to_string());
    if !visitor(&path, &mut build_kind, false) {
        return false;
    }
    let path = format!("{prefix}.point_count");
    let mut build_count = || ScenePropertyValue::Unsigned(points.len() as u64);
    if !visitor(&path, &mut build_count, false) {
        return false;
    }
    for (index, point) in points.iter().enumerate() {
        let path = format!("{prefix}.points.{index}");
        let mut build_point = || ScenePropertyValue::Vec3(point.to_array());
        if !visitor(&path, &mut build_point, false) {
            return false;
        }
    }
    true
}

fn measure(shape: &ColliderShape, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..TRAVERSALS_PER_SAMPLE {
        let mut visitor =
            |path: &str, value: &mut dyn FnMut() -> ScenePropertyValue, animatable: bool| {
                checksum = checksum
                    .wrapping_add(black_box(path.len()))
                    .wrapping_add(usize::from(animatable));
                black_box(value());
                true
            };
        let completed = if optimized {
            visit_collider_shape_property_entries(black_box(shape), "Collider.shape", &mut visitor)
        } else {
            legacy_visit_convex_hull(black_box(shape), "Collider.shape", &mut visitor)
        };
        black_box(completed);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
