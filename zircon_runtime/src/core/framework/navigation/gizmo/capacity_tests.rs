use std::hint::black_box;
use std::time::Instant;

use super::{
    navigation_gizmo_line_capacity, NavigationGizmoLink, NavigationGizmoSnapshot,
    NavigationGizmoTriangle, OverlayLineSegment, OverlayPickShape, Vec3, Vec4, AREA_JUMP,
    AREA_WALKABLE,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 1_024;
const TRIANGLES_PER_BUILD: usize = 128;
const LINKS_PER_BUILD: usize = 128;
const LINES_PER_BUILD: usize = TRIANGLES_PER_BUILD * 3 + LINKS_PER_BUILD;

#[test]
fn optimization_batch_20260826es_runtime188_capacity_preserves_navigation_overlay() {
    let snapshot = NavigationGizmoSnapshot {
        triangles: (0..TRIANGLES_PER_BUILD)
            .map(|index| NavigationGizmoTriangle {
                vertices: [
                    [index as f32, 0.0, 0.0],
                    [index as f32 + 1.0, 0.0, 0.0],
                    [index as f32, 0.0, 1.0],
                ],
                area: AREA_WALKABLE,
                tile: index as u32,
            })
            .collect(),
        off_mesh_links: (0..LINKS_PER_BUILD)
            .map(|index| NavigationGizmoLink {
                start: [index as f32, 0.0, 0.0],
                end: [index as f32, 0.0, 1.0],
                area: AREA_JUMP,
                bidirectional: index % 2 == 0,
            })
            .collect(),
    };

    let overlay = snapshot.to_scene_gizmo_overlay(42, true);

    assert_eq!(overlay.lines.len(), LINES_PER_BUILD);
    assert!(overlay.lines.capacity() >= LINES_PER_BUILD);
    assert_eq!(overlay.pick_shapes.len(), LINKS_PER_BUILD);
    assert!(overlay.pick_shapes.capacity() >= LINKS_PER_BUILD);
    assert_eq!(overlay.owner, 42);
    assert!(overlay.selected);
    assert_eq!(navigation_gizmo_line_capacity(0, 0), 0);
    assert_eq!(
        navigation_gizmo_line_capacity(TRIANGLES_PER_BUILD, LINKS_PER_BUILD),
        LINES_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826es_runtime188_navigation_overlay_reserves_exact_outputs() {
    let source = include_str!("../gizmo.rs");
    assert!(source.contains("Vec::with_capacity(navigation_gizmo_line_capacity("));
    assert!(source.contains("self.triangles.len()"));
    assert!(source.contains("self.off_mesh_links.len()"));
    assert!(source.contains("Vec::with_capacity(self.off_mesh_links.len())"));
    assert!(source.contains("triangle_count.saturating_mul(3)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826es_runtime188_navigation_gizmo_overlay_capacity_bench() {
    let line = OverlayLineSegment {
        start: Vec3::from_array([0.0, 0.0, 0.0]),
        end: Vec3::from_array([1.0, 0.0, 1.0]),
        color: Vec4::new(0.15, 0.78, 0.42, 0.9),
    };
    let pick_shape = OverlayPickShape::Segment {
        start: Vec3::from_array([0.0, 0.0, 0.0]),
        end: Vec3::from_array([1.0, 0.0, 1.0]),
        thickness: 0.08,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&line, &pick_shape, false));
            optimized_samples.push(measure(&line, &pick_shape, true));
        } else {
            optimized_samples.push(measure(&line, &pick_shape, true));
            legacy_samples.push(measure(&line, &pick_shape, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME188_NAVIGATION_GIZMO_OVERLAY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} triangles_per_build={TRIANGLES_PER_BUILD} \
links_per_build={LINKS_PER_BUILD} lines_per_build={LINES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(line: &OverlayLineSegment, pick_shape: &OverlayPickShape, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut lines = if reserve {
            Vec::with_capacity(LINES_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut pick_shapes = if reserve {
            Vec::with_capacity(LINKS_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..LINES_PER_BUILD {
            lines.push(black_box(line.clone()));
        }
        for _ in 0..LINKS_PER_BUILD {
            pick_shapes.push(black_box(pick_shape.clone()));
        }
        checksum ^=
            black_box(lines.len() ^ lines.capacity() ^ pick_shapes.len() ^ pick_shapes.capacity());
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
