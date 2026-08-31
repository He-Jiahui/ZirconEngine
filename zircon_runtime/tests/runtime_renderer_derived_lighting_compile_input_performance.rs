use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    AdvancedPbrMaterialFrameUsage, CameraRenderDescriptor, EnvironmentExtract, FallbackSkyboxKind,
    PostProcessVolumeExtract, PreviewEnvironmentExtract, ProbeInfluenceShape, ReflectionProbeData,
    RenderCameraOrderInput, RenderDirectionalLightSnapshot, RenderFrameExtract, RenderLayerSet,
    RenderOverlayExtract, RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
    RenderSceneGeometryExtract, RenderWorldSnapshotHandle, SceneViewportRenderPacket,
    SourceCubemapEnvironment, SourceCubemapMipChain, SubsurfaceProfileData, sort_render_cameras,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Quat, Vec3};

const MID_LIGHT_COUNT: usize = 1_000;
const LARGE_LIGHT_COUNT: usize = 10_000;
const WARMUP_COUNT: usize = 3;
const SAMPLE_COUNT: usize = 17;
const CAMERA_SUBMISSION_COUNT: usize = 4;

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
struct ProfileSummary {
    allocation_count_p50: u64,
    requested_bytes_p50: u64,
    copied_domain_bytes_p50: u64,
    peak_live_bytes_p50: u64,
    elapsed_p50_ns: u64,
    elapsed_p95_ns: u64,
}

struct DerivedLightingCompileInputStorage {
    material_features: AdvancedPbrMaterialFrameUsage,
    subsurface_profiles: Arc<[SubsurfaceProfileData]>,
    subsurface_material_profile_indices: Arc<[u32]>,
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn renderer_owned_lighting_compile_inputs_avoid_shared_lighting_domain_cow() {
    for light_count in [1, MID_LIGHT_COUNT, LARGE_LIGHT_COUNT] {
        let extract = render_frame_extract_with_lights(light_count);
        let legacy = profile_legacy_shared_lighting_write(&extract, light_count);
        let renderer_owned = profile_renderer_owned_input_storage();
        print_profile("legacy_shared_lighting_write", light_count, legacy);
        print_profile("renderer_owned_compile_input", light_count, renderer_owned);

        assert_eq!(renderer_owned.copied_domain_bytes_p50, 0);
        assert!(renderer_owned.requested_bytes_p50 < 1_024);
        assert!(renderer_owned.allocation_count_p50 <= 2);
        if light_count == LARGE_LIGHT_COUNT {
            assert!(legacy.copied_domain_bytes_p50 > 0);
            assert!(legacy.requested_bytes_p50 >= legacy.copied_domain_bytes_p50);
            assert!(renderer_owned.elapsed_p50_ns < legacy.elapsed_p50_ns);
        }
    }
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn renderer_owned_environment_source_override_avoids_shared_environment_domain_cow() {
    let hydrated_source = source_cubemap_environment();
    for probe_count in [1, MID_LIGHT_COUNT, LARGE_LIGHT_COUNT] {
        let extract = render_frame_extract_with_probes(probe_count);
        let legacy = profile_legacy_shared_environment_write(
            &extract,
            hydrated_source.as_ref(),
            probe_count,
        );
        let renderer_owned = profile_renderer_owned_environment_override(&hydrated_source);
        print_environment_profile("legacy_shared_environment_write", probe_count, legacy);
        print_environment_profile(
            "renderer_owned_environment_override",
            probe_count,
            renderer_owned,
        );

        assert_eq!(renderer_owned.copied_domain_bytes_p50, 0);
        assert_eq!(renderer_owned.allocation_count_p50, 0);
        assert_eq!(renderer_owned.requested_bytes_p50, 0);
        if probe_count == LARGE_LIGHT_COUNT {
            assert!(legacy.copied_domain_bytes_p50 > 0);
            assert!(legacy.requested_bytes_p50 >= legacy.copied_domain_bytes_p50);
            assert!(renderer_owned.elapsed_p50_ns < legacy.elapsed_p50_ns);
        }
    }
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn renderer_owned_particle_history_avoids_shared_particle_domain_cow() {
    for particle_count in [1, MID_LIGHT_COUNT, LARGE_LIGHT_COUNT] {
        let extract = render_frame_extract_with_particles(particle_count);
        let previous_sprites =
            vec![RenderParticlePreviousSpriteSnapshot::default(); particle_count];
        let legacy = profile_legacy_shared_particle_history_write(
            &extract,
            &previous_sprites,
            particle_count,
        );
        let renderer_owned = profile_renderer_owned_particle_history(&previous_sprites);
        print_particle_profile(
            "legacy_shared_particle_history_write",
            particle_count,
            legacy,
        );
        print_particle_profile(
            "renderer_owned_particle_history",
            particle_count,
            renderer_owned,
        );

        assert_eq!(renderer_owned.copied_domain_bytes_p50, 0);
        assert_eq!(renderer_owned.allocation_count_p50, 0);
        assert_eq!(renderer_owned.requested_bytes_p50, 0);
        if particle_count == LARGE_LIGHT_COUNT {
            assert!(legacy.copied_domain_bytes_p50 > 0);
            assert!(legacy.requested_bytes_p50 >= legacy.copied_domain_bytes_p50);
            assert!(renderer_owned.elapsed_p50_ns < legacy.elapsed_p50_ns);
        }
    }
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn renderer_owned_post_process_snapshot_avoids_camera_loop_source_clones() {
    for volume_count in [1, MID_LIGHT_COUNT, LARGE_LIGHT_COUNT] {
        let extract = render_frame_extract_with_post_process_volumes(volume_count);
        let renderer_snapshot = Arc::new(extract.post_process.clone());
        let legacy = profile_legacy_camera_loop_post_process_clones(&extract, volume_count);
        let renderer_owned = profile_renderer_owned_post_process_sharing(&renderer_snapshot);
        print_post_process_profile(
            "legacy_camera_loop_post_process_clone",
            volume_count,
            legacy,
        );
        print_post_process_profile(
            "renderer_owned_post_process_snapshot",
            volume_count,
            renderer_owned,
        );

        assert_eq!(renderer_owned.copied_domain_bytes_p50, 0);
        assert_eq!(renderer_owned.allocation_count_p50, 0);
        assert_eq!(renderer_owned.requested_bytes_p50, 0);
        if volume_count == LARGE_LIGHT_COUNT {
            assert!(legacy.copied_domain_bytes_p50 > 0);
            assert!(legacy.requested_bytes_p50 >= legacy.copied_domain_bytes_p50);
            assert!(renderer_owned.elapsed_p50_ns < legacy.elapsed_p50_ns);
        }
    }
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn camera_submission_projection_avoids_full_view_camera_clones() {
    for camera_count in [1, MID_LIGHT_COUNT, LARGE_LIGHT_COUNT] {
        let (extract, selected) = render_frame_extract_with_cameras(camera_count);
        let legacy = profile_legacy_camera_submission_projection(&extract, &selected, camera_count);
        let projected = profile_single_camera_submission_projection(&extract, &selected);
        print_camera_projection_profile("legacy_full_view_clone", camera_count, legacy);
        print_camera_projection_profile("single_camera_projection", camera_count, projected);

        assert!(projected.allocation_count_p50 <= 2);
        assert!(
            projected.copied_domain_bytes_p50
                <= std::mem::size_of::<CameraRenderDescriptor>() as u64
        );
        if camera_count == LARGE_LIGHT_COUNT {
            assert!(legacy.requested_bytes_p50 > projected.requested_bytes_p50);
            assert!(legacy.copied_domain_bytes_p50 > projected.copied_domain_bytes_p50);
            assert!(projected.elapsed_p50_ns < legacy.elapsed_p50_ns);
        }
    }
}

fn profile_legacy_shared_lighting_write(
    extract: &RenderFrameExtract,
    light_count: usize,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        let mut submission = extract.clone();
        submission.lighting.advanced_lighting.material_features = material_usage();
        black_box(submission);
    }

    profile_samples(|| {
        let mut submission = extract.clone();
        begin_profile();
        let started = Instant::now();
        submission.lighting.advanced_lighting.material_features = material_usage();
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission);
        (
            allocations,
            elapsed,
            light_count.saturating_mul(std::mem::size_of::<RenderDirectionalLightSnapshot>())
                as u64,
        )
    })
}

fn profile_renderer_owned_input_storage() -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        black_box(renderer_owned_inputs());
    }

    profile_samples(|| {
        begin_profile();
        let started = Instant::now();
        let inputs = renderer_owned_inputs();
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&inputs);
        (allocations, elapsed, 0)
    })
}

fn profile_legacy_shared_environment_write(
    extract: &RenderFrameExtract,
    hydrated_source: &SourceCubemapEnvironment,
    probe_count: usize,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        let mut submission = extract.clone();
        submission.environment.skybox.source_cubemap = Some(hydrated_source.clone());
        black_box(submission);
    }

    profile_samples(|| {
        let mut submission = extract.clone();
        begin_profile();
        let started = Instant::now();
        submission.environment.skybox.source_cubemap = Some(hydrated_source.clone());
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission);
        (
            allocations,
            elapsed,
            probe_count.saturating_mul(std::mem::size_of::<ReflectionProbeData>()) as u64,
        )
    })
}

fn profile_renderer_owned_environment_override(
    hydrated_source: &SourceCubemapEnvironment,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        black_box(hydrated_source.clone());
    }

    profile_samples(|| {
        begin_profile();
        let started = Instant::now();
        let override_source = hydrated_source.clone();
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&override_source);
        (allocations, elapsed, 0)
    })
}

fn profile_legacy_shared_particle_history_write(
    extract: &RenderFrameExtract,
    previous_sprites: &[RenderParticlePreviousSpriteSnapshot],
    particle_count: usize,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        let mut submission = extract.clone();
        let previous_sprites = previous_sprites.to_vec();
        submission.particles.previous_sprites = previous_sprites;
        black_box(submission);
    }

    profile_samples(|| {
        let mut submission = extract.clone();
        begin_profile();
        let started = Instant::now();
        let previous_sprites = previous_sprites.to_vec();
        submission.particles.previous_sprites = previous_sprites;
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission);
        (
            allocations,
            elapsed,
            particle_count.saturating_mul(
                std::mem::size_of::<RenderParticleSpriteSnapshot>()
                    + std::mem::size_of::<RenderParticlePreviousSpriteSnapshot>(),
            ) as u64,
        )
    })
}

fn profile_renderer_owned_particle_history(
    previous_sprites: &[RenderParticlePreviousSpriteSnapshot],
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        let mut viewport_history = previous_sprites.to_vec();
        black_box(std::mem::take(&mut viewport_history));
    }

    profile_samples(|| {
        let mut viewport_history = previous_sprites.to_vec();
        begin_profile();
        let started = Instant::now();
        let submission_history = std::mem::take(&mut viewport_history);
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission_history);
        (allocations, elapsed, 0)
    })
}

fn profile_legacy_camera_loop_post_process_clones(
    extract: &RenderFrameExtract,
    volume_count: usize,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        for _ in 0..CAMERA_SUBMISSION_COUNT {
            black_box(extract.post_process.clone());
        }
    }

    profile_samples(|| {
        begin_profile();
        let started = Instant::now();
        for _ in 0..CAMERA_SUBMISSION_COUNT {
            black_box(extract.post_process.clone());
        }
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        (
            allocations,
            elapsed,
            volume_count
                .saturating_mul(CAMERA_SUBMISSION_COUNT)
                .saturating_mul(std::mem::size_of::<PostProcessVolumeExtract>()) as u64,
        )
    })
}

fn profile_renderer_owned_post_process_sharing(
    snapshot: &Arc<zircon_runtime::core::framework::render::PostProcessExtract>,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        for _ in 0..CAMERA_SUBMISSION_COUNT {
            black_box(Arc::clone(snapshot));
        }
    }

    profile_samples(|| {
        begin_profile();
        let started = Instant::now();
        for _ in 0..CAMERA_SUBMISSION_COUNT {
            black_box(Arc::clone(snapshot));
        }
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        (allocations, elapsed, 0)
    })
}

fn profile_legacy_camera_submission_projection(
    extract: &RenderFrameExtract,
    selected: &CameraRenderDescriptor,
    camera_count: usize,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        black_box(
            extract
                .clone()
                .with_selected_camera_descriptor(selected.clone()),
        );
    }

    profile_samples(|| {
        let selected = selected.clone();
        begin_profile();
        let started = Instant::now();
        let submission = extract.clone().with_selected_camera_descriptor(selected);
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission);
        (
            allocations,
            elapsed,
            camera_count.saturating_mul(std::mem::size_of::<CameraRenderDescriptor>()) as u64,
        )
    })
}

fn profile_single_camera_submission_projection(
    extract: &RenderFrameExtract,
    selected: &CameraRenderDescriptor,
) -> ProfileSummary {
    for _ in 0..WARMUP_COUNT {
        black_box(extract.for_camera_submission(selected.clone()));
    }

    profile_samples(|| {
        let selected = selected.clone();
        begin_profile();
        let started = Instant::now();
        let submission = extract.for_camera_submission(selected);
        let elapsed = started.elapsed().as_nanos() as u64;
        let allocations = finish_profile();
        black_box(&submission);
        (
            allocations,
            elapsed,
            std::mem::size_of::<CameraRenderDescriptor>() as u64,
        )
    })
}

fn profile_samples(
    mut operation: impl FnMut() -> (AllocationSnapshot, u64, u64),
) -> ProfileSummary {
    let mut elapsed_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut allocation_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut requested_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut peak_live_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut copied_lighting_byte_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let (allocations, elapsed, copied_lighting_bytes) = operation();
        elapsed_samples.push(elapsed);
        allocation_samples.push(allocations.allocation_count);
        requested_byte_samples.push(allocations.requested_bytes);
        peak_live_byte_samples.push(allocations.peak_live_bytes);
        copied_lighting_byte_samples.push(copied_lighting_bytes);
    }

    elapsed_samples.sort_unstable();
    allocation_samples.sort_unstable();
    requested_byte_samples.sort_unstable();
    peak_live_byte_samples.sort_unstable();
    copied_lighting_byte_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    ProfileSummary {
        allocation_count_p50: allocation_samples[p50],
        requested_bytes_p50: requested_byte_samples[p50],
        copied_domain_bytes_p50: copied_lighting_byte_samples[p50],
        peak_live_bytes_p50: peak_live_byte_samples[p50],
        elapsed_p50_ns: elapsed_samples[p50],
        elapsed_p95_ns: elapsed_samples[p95],
    }
}

fn renderer_owned_inputs() -> DerivedLightingCompileInputStorage {
    DerivedLightingCompileInputStorage {
        material_features: material_usage(),
        subsurface_profiles: Vec::<SubsurfaceProfileData>::new().into(),
        subsurface_material_profile_indices: Vec::<u32>::new().into(),
    }
}

fn material_usage() -> AdvancedPbrMaterialFrameUsage {
    AdvancedPbrMaterialFrameUsage {
        late_forward_opaque: true,
        ..Default::default()
    }
}

fn render_frame_extract_with_lights(light_count: usize) -> RenderFrameExtract {
    let light = RenderDirectionalLightSnapshot {
        node_id: 1,
        light_id: 1,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::ONE,
        intensity: 1.0,
        mobility: Mobility::Dynamic,
        shadow: None,
    };
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera: Default::default(),
                meshes: Vec::new(),
                directional_lights: vec![light; light_count],
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
                clear_color: Default::default(),
            },
            virtual_geometry_debug: None,
        },
    )
}

fn render_frame_extract_with_probes(probe_count: usize) -> RenderFrameExtract {
    let mut extract = render_frame_extract_with_lights(0);
    let probe = ReflectionProbeData::try_new(
        1,
        Vec3::ZERO,
        Quat::IDENTITY,
        ProbeInfluenceShape::sphere(1.0, 0.0).expect("valid benchmark probe shape"),
        Vec3::ONE,
    )
    .expect("valid benchmark probe");
    extract.environment.probes = vec![probe; probe_count];
    extract
}

fn render_frame_extract_with_particles(particle_count: usize) -> RenderFrameExtract {
    let mut extract = render_frame_extract_with_lights(0);
    extract.particles.sprites = vec![RenderParticleSpriteSnapshot::default(); particle_count];
    extract
}

fn render_frame_extract_with_post_process_volumes(volume_count: usize) -> RenderFrameExtract {
    let mut extract = render_frame_extract_with_lights(0);
    extract.post_process.volumes = vec![
        PostProcessVolumeExtract::global(
            0.0,
            1.0,
            RenderLayerSet::from_scene_schema_v1_mask(1),
            Vec::new(),
        );
        volume_count
    ];
    extract
}

fn render_frame_extract_with_cameras(
    camera_count: usize,
) -> (RenderFrameExtract, CameraRenderDescriptor) {
    let mut extract = render_frame_extract_with_lights(0);
    let cameras = (0..camera_count)
        .map(|index| {
            CameraRenderDescriptor::from_camera_payload(Some(index as u64 + 1), Default::default())
        })
        .collect::<Vec<_>>();
    let selected = cameras
        .last()
        .expect("camera projection benchmark requires one camera")
        .clone();
    let order_report = sort_render_cameras(cameras.iter().cloned().map(|camera| {
        RenderCameraOrderInput::from_descriptor(
            camera
                .entity
                .expect("camera projection benchmark descriptors carry entities"),
            camera,
        )
    }));
    extract.view = extract
        .view
        .with_cameras(cameras)
        .with_scene_camera_order_report(
            selected
                .entity
                .expect("selected benchmark camera carries an entity"),
            order_report,
        );
    (extract, selected)
}

fn source_cubemap_environment() -> SourceCubemapEnvironment {
    SourceCubemapEnvironment::new(
        SourceCubemapMipChain::new(
            1,
            1,
            vec![[0.25, 0.5, 0.75, 1.0]; 6],
            1,
            1,
            vec![[0.1, 0.2, 0.3, 1.0]; 6],
        ),
        1,
        [1, 2, 3, 4],
    )
}

fn print_profile(operation: &str, light_count: usize, profile: ProfileSummary) {
    println!(
        "RUNTIME07_RENDERER_DERIVED_LIGHTING_INPUT_V1 operation={} lights={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_lighting_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        operation,
        light_count,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_domain_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
}

fn print_environment_profile(operation: &str, probe_count: usize, profile: ProfileSummary) {
    println!(
        "RUNTIME07_RENDERER_DERIVED_ENVIRONMENT_INPUT_V1 operation={} probes={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_environment_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        operation,
        probe_count,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_domain_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
}

fn print_particle_profile(operation: &str, particle_count: usize, profile: ProfileSummary) {
    println!(
        "RUNTIME07_RENDERER_DERIVED_PARTICLE_HISTORY_V1 operation={} particles={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_particle_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        operation,
        particle_count,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_domain_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
}

fn print_post_process_profile(operation: &str, volume_count: usize, profile: ProfileSummary) {
    println!(
        "RUNTIME07_RENDERER_DERIVED_POST_PROCESS_V1 operation={} volumes={} camera_submissions={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_post_process_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        operation,
        volume_count,
        CAMERA_SUBMISSION_COUNT,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_domain_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
}

fn print_camera_projection_profile(operation: &str, camera_count: usize, profile: ProfileSummary) {
    println!(
        "RUNTIME07_CAMERA_SUBMISSION_PROJECTION_V1 operation={} cameras={} samples={} warmups={} allocation_count_p50={} requested_bytes_p50={} copied_camera_descriptor_bytes_p50={} peak_live_bytes_p50={} elapsed_p50_ns={} elapsed_p95_ns={}",
        operation,
        camera_count,
        SAMPLE_COUNT,
        WARMUP_COUNT,
        profile.allocation_count_p50,
        profile.requested_bytes_p50,
        profile.copied_domain_bytes_p50,
        profile.peak_live_bytes_p50,
        profile.elapsed_p50_ns,
        profile.elapsed_p95_ns,
    );
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
