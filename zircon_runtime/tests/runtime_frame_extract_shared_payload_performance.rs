use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSpriteAnchor,
    RenderSpriteImageMode, RenderSpriteSnapshot, RenderWorldSnapshotHandle, RendererCommon,
    SceneViewportRenderPacket,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};

const SCENE_ITEM_COUNT: usize = 10_000;
const MID_SCENE_ITEM_COUNT: usize = 1_000;
const WARMUP_COUNT: usize = 3;
const SAMPLE_COUNT: usize = 17;

struct CountingAllocator;

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;
static PROFILE_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static REQUESTED_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            record_allocation(layout.size() as u64);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            record_allocation(layout.size() as u64);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if PROFILE_ACTIVE.load(Ordering::Relaxed) {
            decrease_live_bytes(layout.size() as u64);
        }
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        if !replacement.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
            REQUESTED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            let old_size = layout.size() as u64;
            let new_size = new_size as u64;
            let live_bytes = if new_size >= old_size {
                LIVE_BYTES
                    .fetch_add(new_size - old_size, Ordering::Relaxed)
                    .saturating_add(new_size - old_size)
            } else {
                decrease_live_bytes(old_size - new_size)
            };
            update_peak_live_bytes(live_bytes);
        }
        replacement
    }
}

#[derive(Clone, Copy)]
struct AllocationSnapshot {
    allocation_count: u64,
    requested_bytes: u64,
    peak_live_bytes: u64,
}

#[derive(Clone, Copy)]
struct CloneProfileSummary {
    allocation_count_p50: u64,
    requested_bytes_p50: u64,
    copied_scene_bytes_p50: u64,
    peak_live_bytes_p50: u64,
    elapsed_p50_ns: u64,
    elapsed_p95_ns: u64,
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn shared_render_frame_extract_cache_clone_does_not_scale_with_scene_cardinality() {
    let single_item_extract = large_render_frame_extract(1);
    let mid_extract = large_render_frame_extract(MID_SCENE_ITEM_COUNT);
    let large_extract = large_render_frame_extract(SCENE_ITEM_COUNT);
    let single_item = profile_clone(&single_item_extract, logical_scene_bytes(1));
    let mid = profile_clone(&mid_extract, logical_scene_bytes(MID_SCENE_ITEM_COUNT));
    let large = profile_clone(&large_extract, logical_scene_bytes(SCENE_ITEM_COUNT));

    print_profile(1, single_item);
    print_profile(MID_SCENE_ITEM_COUNT, mid);
    print_profile(SCENE_ITEM_COUNT, large);

    assert_eq!(single_item.copied_scene_bytes_p50, 0);
    assert_eq!(mid.copied_scene_bytes_p50, 0);
    assert_eq!(large.copied_scene_bytes_p50, 0);
    assert_cardinality_invariant(single_item, mid);
    assert_cardinality_invariant(single_item, large);
    assert!(large.allocation_count_p50 <= 16);
    assert!(large.requested_bytes_p50 < 64 * 1024);
}

fn assert_cardinality_invariant(baseline: CloneProfileSummary, candidate: CloneProfileSummary) {
    assert_eq!(
        candidate.allocation_count_p50, baseline.allocation_count_p50,
        "cache retain/return clone allocations must not scale with scene cardinality"
    );
    assert_eq!(
        candidate.requested_bytes_p50, baseline.requested_bytes_p50,
        "cache retain/return requested bytes must not scale with scene cardinality"
    );
    assert_eq!(
        candidate.peak_live_bytes_p50, baseline.peak_live_bytes_p50,
        "cache retain/return peak live bytes must not scale with scene cardinality"
    );
}

fn profile_clone(extract: &RenderFrameExtract, logical_scene_bytes: u64) -> CloneProfileSummary {
    for _ in 0..WARMUP_COUNT {
        black_box(extract.clone());
    }

    let mut elapsed_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut allocation_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut requested_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut peak_live_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut copied_scene_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        begin_profile();
        let started = Instant::now();
        let cloned = black_box(extract.clone());
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        let copied_scene_bytes = if extract.shares_scene_with(&cloned) {
            0
        } else {
            logical_scene_bytes
        };
        black_box(&cloned);
        drop(cloned);

        elapsed_samples.push(elapsed);
        allocation_samples.push(allocations.allocation_count);
        requested_byte_samples.push(allocations.requested_bytes);
        peak_live_byte_samples.push(allocations.peak_live_bytes);
        copied_scene_byte_samples.push(copied_scene_bytes);
    }

    elapsed_samples.sort_unstable();
    allocation_samples.sort_unstable();
    requested_byte_samples.sort_unstable();
    peak_live_byte_samples.sort_unstable();
    copied_scene_byte_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;

    CloneProfileSummary {
        allocation_count_p50: allocation_samples[p50],
        requested_bytes_p50: requested_byte_samples[p50],
        copied_scene_bytes_p50: copied_scene_byte_samples[p50],
        peak_live_bytes_p50: peak_live_byte_samples[p50],
        elapsed_p50_ns: elapsed_samples[p50],
        elapsed_p95_ns: elapsed_samples[p95],
    }
}

fn print_profile(item_count: usize, profile: CloneProfileSummary) {
    println!(
        "RUNTIME07_RENDER_FRAME_EXTRACT_SHARED_V3 cache_clone_operation=retain_or_return meshes={} lights={} sprites={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_scene_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        item_count,
        item_count,
        item_count,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_scene_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
}

fn logical_scene_bytes(item_count: usize) -> u64 {
    item_count.saturating_mul(
        std::mem::size_of::<RenderMeshSnapshot>()
            + std::mem::size_of::<RenderDirectionalLightSnapshot>()
            + std::mem::size_of::<RenderSpriteSnapshot>(),
    ) as u64
}

fn large_render_frame_extract(item_count: usize) -> RenderFrameExtract {
    let model = ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
        "runtime07/profile/model",
    ));
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "runtime07/profile/material",
    ));
    let image = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "runtime07/profile/image",
    ));
    let layer_mask = RenderLayerSet::from_scene_schema_v1_mask(1);
    let common = RendererCommon {
        layer_mask: layer_mask.clone(),
        ..RendererCommon::default()
    };
    let mesh = RenderMeshSnapshot {
        node_id: 1,
        stable_instance_key: 1 << 16,
        transform_revision: 0,
        transform: Transform::default(),
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: common.clone(),
    };
    let light = RenderDirectionalLightSnapshot {
        node_id: 2,
        light_id: 2,
        layer_mask,
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::ONE,
        intensity: 1.0,
        mobility: Mobility::Dynamic,
        shadow: None,
    };
    let sprite = RenderSpriteSnapshot {
        entity: 3,
        transform: Transform::default(),
        image,
        material: None,
        atlas_region: None,
        rect: None,
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::CENTER,
        custom_size: None,
        image_mode: RenderSpriteImageMode::default(),
        color: Vec4::ONE,
        z_order: 0,
        common,
        material_alpha_mode: RenderMaterialAlphaMode::Opaque,
    };
    let packet = SceneViewportRenderPacket {
        scene: RenderSceneGeometryExtract {
            camera: Default::default(),
            meshes: vec![mesh; item_count],
            directional_lights: vec![light; item_count],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract = RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), packet);
    extract.sprites.sprites = vec![sprite; item_count];
    extract
}

fn begin_profile() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    LIVE_BYTES.store(0, Ordering::Relaxed);
    PEAK_LIVE_BYTES.store(0, Ordering::Relaxed);
    assert!(!PROFILE_ACTIVE.swap(true, Ordering::SeqCst));
}

fn finish_profile() -> AllocationSnapshot {
    assert!(PROFILE_ACTIVE.swap(false, Ordering::SeqCst));
    AllocationSnapshot {
        allocation_count: ALLOCATION_COUNT.load(Ordering::Relaxed),
        requested_bytes: REQUESTED_BYTES.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE_BYTES.load(Ordering::Relaxed),
    }
}

fn record_allocation(size: u64) {
    ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    REQUESTED_BYTES.fetch_add(size, Ordering::Relaxed);
    let live_bytes = LIVE_BYTES
        .fetch_add(size, Ordering::Relaxed)
        .saturating_add(size);
    update_peak_live_bytes(live_bytes);
}

fn decrease_live_bytes(size: u64) -> u64 {
    LIVE_BYTES
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            Some(current.saturating_sub(size))
        })
        .unwrap_or_default()
        .saturating_sub(size)
}

fn update_peak_live_bytes(live_bytes: u64) {
    let mut peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    while live_bytes > peak {
        match PEAK_LIVE_BYTES.compare_exchange_weak(
            peak,
            live_bytes,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }
}
