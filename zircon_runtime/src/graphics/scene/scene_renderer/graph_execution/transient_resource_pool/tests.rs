use crate::graphics::backend::RenderBackend;
use crate::render_graph::{CompiledRenderGraph, QueueLane, RenderGraphBuilder};
use crate::rhi::{BufferUsage, TextureUsage};

use super::super::render_graph_execution_resources::RenderGraphExecutionResources;
use super::*;

mod device_epoch;

#[test]
fn transient_resource_pool_uses_the_shared_graph_device_epoch_owner() {
    let source = include_str!("../transient_resource_pool.rs");
    let allocation = include_str!("allocation.rs");

    assert!(source.contains("active_device_epoch: Option<RenderPassDeviceEpoch>"));
    assert!(source.contains("RenderPassDeviceEpoch::from_profile(device_profile)"));
    assert!(source.contains("let (device_id, generation) = epoch.raw_parts();"));
    assert!(!source.contains("TransientResourcePoolDeviceEpoch"));
    assert!(allocation.contains("epoch: RenderPassDeviceEpoch"));
    assert!(!allocation.contains("TransientResourcePoolDeviceEpoch"));
}

#[test]
fn transient_resource_pool_rejects_texture_descriptor_size_overflow() {
    let desc = TextureDesc::new(
        "oversized-transient-pool-texture",
        u32::MAX,
        u32::MAX,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    );

    assert_eq!(texture_desc_pool_size_bytes(&desc), None);
}

#[test]
fn transient_texture_key_includes_the_declared_view_format_set() {
    let base = TextureDesc::new(
        "view-format-base",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let srgb = base
        .clone()
        .with_view_formats([TextureFormat::Rgba8UnormSrgb]);
    let reordered = base
        .clone()
        .with_view_formats([TextureFormat::Bgra8UnormSrgb, TextureFormat::Rgba8UnormSrgb]);
    let same_set = base
        .clone()
        .with_view_formats([TextureFormat::Rgba8UnormSrgb, TextureFormat::Bgra8UnormSrgb]);

    assert_ne!(
        TransientTextureKey::from(&base),
        TransientTextureKey::from(&srgb)
    );
    assert_eq!(
        TransientTextureKey::from(&reordered),
        TransientTextureKey::from(&same_set),
        "view format order must not split physically compatible pool buckets"
    );
}

#[test]
fn transient_resource_pool_materialization_budget_eviction_orders_candidates_once() {
    struct Candidate {
        label: &'static str,
        last_used_frame: u64,
        byte_size: u64,
    }

    let mut pool = BTreeMap::from([
        (
            0_u8,
            vec![
                Candidate {
                    label: "newest-a",
                    last_used_frame: 3,
                    byte_size: 10,
                },
                Candidate {
                    label: "oldest-a",
                    last_used_frame: 0,
                    byte_size: 10,
                },
                Candidate {
                    label: "newer-a",
                    last_used_frame: 2,
                    byte_size: 10,
                },
                Candidate {
                    label: "older-a",
                    last_used_frame: 1,
                    byte_size: 10,
                },
            ],
        ),
        (
            1_u8,
            vec![Candidate {
                label: "oldest-b",
                last_used_frame: 0,
                byte_size: 10,
            }],
        ),
    ]);

    let (evicted, retained_count, retained_bytes, accounted_count, sort_candidate_count) =
        evict_pool_to_budget(&mut pool, 20, |entry| {
            (entry.last_used_frame, entry.byte_size)
        });

    assert_eq!((evicted, retained_count, retained_bytes), (3, 2, 20));
    assert_eq!((accounted_count, sort_candidate_count), (5, 5));
    assert!(!pool.contains_key(&1));
    assert_eq!(
        pool.get(&0)
            .unwrap()
            .iter()
            .map(|entry| entry.label)
            .collect::<Vec<_>>(),
        vec!["newest-a", "newer-a"]
    );

    let (evicted, retained_count, retained_bytes, accounted_count, sort_candidate_count) =
        evict_pool_to_budget(&mut pool, 10, |entry| {
            (entry.last_used_frame, entry.byte_size)
        });
    assert_eq!((evicted, retained_count, retained_bytes), (1, 1, 10));
    assert_eq!((accounted_count, sort_candidate_count), (2, 2));
    assert_eq!(pool.get(&0).unwrap()[0].label, "newest-a");
}

#[test]
fn transient_resource_pool_budget_eviction_accounts_for_saturated_resource_sizes() {
    struct Candidate {
        last_used_frame: u64,
        byte_size: u64,
    }

    let mut pool = BTreeMap::from([(
        0_u8,
        vec![
            Candidate {
                last_used_frame: 0,
                byte_size: u64::MAX,
            },
            Candidate {
                last_used_frame: 1,
                byte_size: u64::MAX,
            },
        ],
    )]);

    let (evicted, retained_count, retained_bytes, accounted_count, sort_candidate_count) =
        evict_pool_to_budget(&mut pool, 0, |entry| {
            (entry.last_used_frame, entry.byte_size)
        });

    assert_eq!((evicted, retained_count, retained_bytes), (2, 0, 0));
    assert_eq!((accounted_count, sort_candidate_count), (2, 2));
    assert!(pool.is_empty());
}

#[test]
fn transient_resource_pool_byte_reporting_saturates_instead_of_panicking() {
    assert_eq!(saturating_pool_byte_count(u128::from(u64::MAX)), u64::MAX);
    assert_eq!(
        saturating_pool_byte_count(u128::from(u64::MAX) + 1),
        u64::MAX
    );
}

#[test]
fn transient_resource_pool_reports_persistent_extraction_cold_and_warm_costs() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let desc = TextureDesc::new(
        "persistent-history-source",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let cold_allocation = pool
        .acquire_persistent_texture(&backend.device, &desc)
        .unwrap();
    let cold_identity = cold_allocation.identity();
    pool.release_texture(cold_allocation);
    pool.end_frame();
    let cold = pool.last_frame_report();
    assert_eq!(cold.persistent_texture_request_count, 1);
    assert_eq!(cold.persistent_texture_requested_bytes, 4_096);
    assert_eq!(cold.persistent_texture_created_count, 1);
    assert_eq!(cold.persistent_texture_reused_count, 0);

    pool.begin_frame(backend.device_profile());
    let warm_allocation = pool
        .acquire_persistent_texture(&backend.device, &desc)
        .unwrap();
    let warm_identity = warm_allocation.identity();
    assert_eq!(warm_identity, cold_identity);
    pool.release_texture(warm_allocation);
    pool.end_frame();
    let warm = pool.last_frame_report();
    assert_eq!(warm.persistent_texture_request_count, 1);
    assert_eq!(warm.persistent_texture_requested_bytes, 4_096);
    assert_eq!(warm.persistent_texture_created_count, 0);
    assert_eq!(warm.persistent_texture_reused_count, 1);
}

#[test]
fn transient_resource_pool_evicts_oldest_entries_to_budget() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let texture_desc = TextureDesc::new(
        "budgeted-color",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let buffer_desc = BufferDesc::new(
        "budgeted-buffer",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    );
    let mut pool = TransientResourcePool::with_budgets(4_096, 64);

    pool.begin_frame(backend.device_profile());
    let first_texture = pool
        .acquire_texture(&backend.device, &texture_desc)
        .unwrap();
    let second_texture = pool
        .acquire_texture(&backend.device, &texture_desc)
        .unwrap();
    pool.release_texture(first_texture);
    pool.release_texture(second_texture);
    let first_buffer = pool.acquire_buffer(&backend.device, &buffer_desc).unwrap();
    let second_buffer = pool.acquire_buffer(&backend.device, &buffer_desc).unwrap();
    pool.release_buffer(first_buffer);
    pool.release_buffer(second_buffer);
    pool.end_frame();

    let report = pool.last_frame_report();
    assert_eq!(report.texture_created_count, 2);
    assert_eq!(report.buffer_created_count, 2);
    assert_eq!(report.budget_evicted_texture_count, 1);
    assert_eq!(report.budget_evicted_buffer_count, 1);
    assert_eq!(report.stale_texture_scan_count, 2);
    assert_eq!(report.stale_buffer_scan_count, 2);
    assert_eq!(report.budget_texture_accounted_count, 2);
    assert_eq!(report.budget_buffer_accounted_count, 2);
    assert_eq!(report.budget_texture_sort_candidate_count, 2);
    assert_eq!(report.budget_buffer_sort_candidate_count, 2);
    assert_eq!(report.evicted_texture_count, 0);
    assert_eq!(report.evicted_buffer_count, 0);
    assert_eq!(report.texture_pool_entry_count, 1);
    assert_eq!(report.buffer_pool_entry_count, 1);
    assert_eq!(report.texture_pool_retained_bytes, 4_096);
    assert_eq!(report.buffer_pool_retained_bytes, 64);
    assert_eq!(report.texture_pool_budget_bytes, 4_096);
    assert_eq!(report.buffer_pool_budget_bytes, 64);
}

#[test]
fn transient_resource_pool_evicts_stale_entries_after_keep_frames() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let desc = BufferDesc::new(
        "pooled-buffer",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let buffer = pool.acquire_buffer(&backend.device, &desc).unwrap();
    pool.release_buffer(buffer);
    pool.end_frame();

    for _ in 0..TRANSIENT_RESOURCE_POOL_KEEP_FRAMES {
        pool.begin_frame(backend.device_profile());
        pool.end_frame();
    }

    assert_eq!(pool.last_frame_report().evicted_buffer_count, 1);
    assert_eq!(pool.last_frame_report().buffer_pool_entry_count, 0);
    assert_eq!(pool.last_frame_report().buffer_pool_retained_bytes, 0);
}

#[test]
fn render_post_dynamic_resolution_scale_swap_releases_pool() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut pool = TransientResourcePool::default();
    let half_resolution = dynamic_resolution_frame_graph("render-scale-0-5", 160, 120);
    let full_resolution = dynamic_resolution_frame_graph("render-scale-1-0", 320, 240);

    let first_half = materialize_graph_frame(&backend, &mut pool, &half_resolution);
    assert_eq!(first_half.texture_created_count, 1);
    assert_eq!(first_half.texture_reused_count, 0);
    assert_eq!(first_half.texture_pool_entry_count, 1);

    let full = materialize_graph_frame(&backend, &mut pool, &full_resolution);
    assert_eq!(full.texture_created_count, 1);
    assert_eq!(full.texture_reused_count, 0);
    assert_eq!(
        full.texture_pool_entry_count, 2,
        "switching from render_scale 0.5 to 1.0 should retain only the two live scale buckets"
    );

    let second_half = materialize_graph_frame(&backend, &mut pool, &half_resolution);
    assert_eq!(
        second_half.texture_created_count, 0,
        "returning to render_scale 0.5 must reuse the compatible half-size backing"
    );
    assert_eq!(second_half.texture_reused_count, 1);
    assert_eq!(
        second_half.texture_pool_entry_count, 2,
        "scale toggling must not grow the pool beyond the distinct descriptor buckets"
    );
}

fn dynamic_resolution_frame_graph(label: &str, width: u32, height: u32) -> CompiledRenderGraph {
    let mut builder = RenderGraphBuilder::new(label);
    let scene_color = builder.create_texture(TextureDesc::new(
        "scene-color",
        width,
        height,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let write = builder.add_pass(format!("{label}-write"), QueueLane::Graphics);
    let present = builder.add_pass(format!("{label}-present"), QueueLane::Graphics);
    builder.write_texture(write, scene_color).unwrap();
    builder.read_texture(present, scene_color).unwrap();
    builder.write_external(present, output).unwrap();
    builder.add_dependency(write, present).unwrap();
    builder.compile().unwrap()
}

fn materialize_graph_frame(
    backend: &RenderBackend,
    pool: &mut TransientResourcePool,
    graph: &CompiledRenderGraph,
) -> RenderGraphTransientPoolReport {
    let mut resources = RenderGraphExecutionResources::new();

    pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            graph,
            pool,
        )
        .unwrap();
    assert_eq!(
        resources.resource_report().owned_texture_count,
        1,
        "each dynamic-resolution graph frame should need one concrete scene-color backing"
    );
    resources.release_transient_backings_into_pool(pool);
    pool.end_frame();

    pool.last_frame_report()
}
