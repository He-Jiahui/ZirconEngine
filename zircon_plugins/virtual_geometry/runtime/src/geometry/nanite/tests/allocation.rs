use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use zircon_runtime::asset::MeshVertex;
use zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayloadVertex;
use zircon_runtime::core::math::{Vec2, Vec3};

use super::{
    append_triangle_range_vertices, render_page_vertices, CLUSTER_PAYLOAD_MAGIC,
    PAYLOAD_HEADER_WORD_COUNT, PAYLOAD_ITEM_WORD_COUNT, PAYLOAD_VERSION,
};

const CAPACITY_ITEM_COUNT: usize = 65;
const BENCHMARK_ITEM_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn borrowed_payload_decode_clamps_items_and_reserves_exact_vertices() {
    let payload = triangle_payload(CAPACITY_ITEM_COUNT, CAPACITY_ITEM_COUNT as u32);
    let vertices = mesh_vertices(CAPACITY_ITEM_COUNT * 3);
    let indices = (0..vertices.len() as u32).collect::<Vec<_>>();

    let decoded = render_page_vertices(&payload, &vertices, &indices);

    assert_eq!(decoded.len(), CAPACITY_ITEM_COUNT * 3);
    assert_eq!(decoded.capacity(), decoded.len());

    let truncated = triangle_payload(0, u32::MAX);
    let decoded_truncated = render_page_vertices(&truncated, &vertices, &indices);
    assert!(decoded_truncated.is_empty());
    assert_eq!(decoded_truncated.capacity(), 0);
}

#[test]
fn borrowed_exact_capacity_payload_decode_performance_contract() {
    let payload = triangle_payload(BENCHMARK_ITEM_COUNT, BENCHMARK_ITEM_COUNT as u32);
    let vertices = mesh_vertices(BENCHMARK_ITEM_COUNT * 3);
    let indices = (0..vertices.len() as u32).collect::<Vec<_>>();
    let legacy = || {
        black_box(legacy_render_page_vertices(&payload, &vertices, &indices));
    };
    let optimized = || {
        black_box(render_page_vertices(&payload, &vertices, &indices));
    };

    legacy();
    optimized();
    let legacy_capacity = legacy_render_page_vertices(&payload, &vertices, &indices).capacity();
    let optimized_capacity = render_page_vertices(&payload, &vertices, &indices).capacity();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();

    println!(
        "PERF_RESULT plugins17_borrowed_exact_capacity_page_payload items={BENCHMARK_ITEM_COUNT} vertices={} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_payload_word_vec_allocations_per_sample=1 optimized_payload_word_vec_allocations_per_sample=0 legacy_vertex_vec_growth_allocations_min=2 optimized_vertex_vec_growth_allocations=1 legacy_final_capacity={legacy_capacity} optimized_final_capacity={optimized_capacity} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}",
        BENCHMARK_ITEM_COUNT * 3
    );
    assert_eq!(optimized_capacity, BENCHMARK_ITEM_COUNT * 3);
    assert!(optimized_capacity < legacy_capacity);
    assert!(
        optimized_p95 <= legacy_p95,
        "borrowed exact-capacity decode must not regress P95: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
    );
}

fn triangle_payload(actual_item_count: usize, declared_item_count: u32) -> Vec<u8> {
    let mut words = vec![
        CLUSTER_PAYLOAD_MAGIC,
        PAYLOAD_VERSION,
        0,
        0,
        0,
        0,
        declared_item_count,
    ];
    for item_index in 0..actual_item_count as u32 {
        words.extend([0, 0, item_index, 1]);
    }
    words
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>()
}

fn mesh_vertices(count: usize) -> Vec<MeshVertex> {
    (0..count)
        .map(|index| {
            MeshVertex::new(Vec3::new(index as f32, 0.0, 0.0), Vec3::Y, Vec2::ZERO)
                .with_tangent([1.0, 0.0, 0.0, 1.0])
        })
        .collect()
}

fn legacy_render_page_vertices(
    payload: &[u8],
    vertices: &[MeshVertex],
    indices: &[u32],
) -> Vec<RenderVirtualGeometryPagePayloadVertex> {
    let words = payload
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    if words.len() < PAYLOAD_HEADER_WORD_COUNT
        || words[0] != CLUSTER_PAYLOAD_MAGIC
        || words[1] != PAYLOAD_VERSION
    {
        return Vec::new();
    }

    let item_count = words[6] as usize;
    let mut page_vertices = Vec::new();
    for item_index in 0..item_count {
        let item_base = PAYLOAD_HEADER_WORD_COUNT + item_index * PAYLOAD_ITEM_WORD_COUNT;
        let Some(item) = words.get(item_base..item_base + PAYLOAD_ITEM_WORD_COUNT) else {
            continue;
        };
        append_triangle_range_vertices(
            item[2] as usize,
            item[3] as usize,
            vertices,
            indices,
            &mut page_vertices,
        );
    }
    page_vertices
}

fn paired_samples(legacy: impl Fn(), optimized: impl Fn()) -> (Vec<Duration>, Vec<Duration>) {
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&legacy));
            optimized_samples.push(measure(&optimized));
        } else {
            optimized_samples.push(measure(&optimized));
            legacy_samples.push(measure(&legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(run: impl Fn()) -> Duration {
    let started = Instant::now();
    run();
    started.elapsed()
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}
