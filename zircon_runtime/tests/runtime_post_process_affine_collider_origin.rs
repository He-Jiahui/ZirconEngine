#![cfg(all(feature = "graphics", feature = "native-host"))]

//! Check both the Level's real volume evaluation and the effect on the final frame.

#[path = "support/project_asset_runtime.rs"]
mod project_asset_runtime;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use project_asset_runtime::ProjectAssetTestRuntime;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetManager, AssetUri, ProjectAssetManager};
use zircon_runtime::core::framework::render::{
    CapturedFrame, CorePipelineKind, RenderCameraClearColor, RenderCaptureSource,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderVignetteSettings,
    SceneViewportExtractRequest, ViewportRenderSettings, VolumeShapeExtract,
};
use zircon_runtime::core::math::{Quat, Transform, UVec2, Vec3, Vec4};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::components::{
    CameraComponent, ColliderComponent, ColliderShape, NodeKind, PostProcessVolumeComponent,
};
use zircon_runtime::scene::{DefaultLevelManager, EntityId, LevelMetadata, LevelSystem, World};

const SIZE: UVec2 = UVec2::new(128, 96);
const LAYER: u32 = 0b0010;
const OFFSET: Vec3 = Vec3::new(8.0, 0.0, 0.0);
const VIGNETTE: f32 = 0.75;
const FRAMES: [Sample; 3] = [Sample::Outside, Sample::Rigid, Sample::Shear];

#[derive(Clone, Copy)]
enum Sample {
    Outside,
    Rigid,
    Shear,
}

impl Sample {
    fn label(self) -> &'static str {
        match self {
            Self::Outside => "outside",
            Self::Rigid => "rigid",
            Self::Shear => "shear",
        }
    }
}

// Independent scalar oracle: diag(2,1,1) * Ry(45deg) * (8,0,0,1).
fn analytic_center() -> Vec3 {
    let root_two = std::f32::consts::SQRT_2;
    Vec3::new(8.0 * root_two, 0.0, -4.0 * root_two)
}

struct VolumeLevel {
    _owner: DefaultLevelManager,
    level: LevelSystem,
    camera: EntityId,
    parent: EntityId,
    volume: EntityId,
}

impl VolumeLevel {
    fn new() -> Self {
        let mut world = World::empty();
        let camera = world.spawn_node(NodeKind::Camera).unwrap();
        world
            .insert(
                camera,
                CameraComponent {
                    core_pipeline: CorePipelineKind::Core3d,
                    clear_color: RenderCameraClearColor::Color(Vec4::new(0.68, 0.74, 0.82, 1.0)),
                    ..CameraComponent::default()
                },
            )
            .unwrap();
        world.set_active_camera(camera);
        world.set_render_layer_mask(camera, LAYER).unwrap();
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                parent,
                Transform {
                    scale: Vec3::new(2.0, 1.0, 1.0),
                    ..Transform::default()
                },
            )
            .unwrap();
        let volume = world.spawn_node(NodeKind::Empty).unwrap();
        world.set_render_layer_mask(volume, LAYER).unwrap();
        // Same authored vignette as the existing post-process transition product test.
        let profile = RenderPostProcessVolumeProfile::default().with_effect_stack(
            RenderPostProcessEffectStackSettings {
                vignette: RenderVignetteSettings {
                    intensity: VIGNETTE,
                    smoothness: 0.45,
                    roundness: 1.0,
                },
                ..Default::default()
            },
        );
        world
            .insert(
                volume,
                PostProcessVolumeComponent::local(0.0, 1.0, 0.0, profile),
            )
            .unwrap();
        let owner = DefaultLevelManager::default();
        let level = owner.create_level(world, LevelMetadata::default());
        Self {
            _owner: owner,
            level,
            camera,
            parent,
            volume,
        }
    }

    fn extract(&self, sample: Sample) -> RenderFrameExtract {
        let center = analytic_center();
        let rigid = matches!(sample, Sample::Rigid);
        let offset = if rigid { Vec3::ZERO } else { OFFSET };
        let camera_position = if matches!(sample, Sample::Outside) {
            center + Vec3::Y * 8.0
        } else {
            center
        };
        let world_id = self.level.with_world_mut(|world| {
            world
                .update_transform(self.camera, Transform::from_translation(camera_position))
                .unwrap();
            world
                .set_parent_checked(self.volume, (!rigid).then_some(self.parent))
                .unwrap();
            world
                .update_transform(
                    self.volume,
                    if rigid {
                        Transform::from_translation(center)
                    } else {
                        Transform {
                            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                            ..Transform::default()
                        }
                    },
                )
                .unwrap();
            world
                .set_collider(
                    self.volume,
                    Some(ColliderComponent {
                        shape: ColliderShape::Sphere { radius: 0.25 },
                        local_transform: Transform::from_translation(offset),
                        sensor: true,
                        ..ColliderComponent::default()
                    }),
                )
                .unwrap();
            let matrix = world.world_matrix(self.volume).unwrap();
            let dot = if rigid { 0.0 } else { 1.5 };
            assert!((matrix.x_axis.dot(matrix.z_axis) - dot).abs() < 1e-5);
            assert!((matrix.transform_point3(offset) - center).length() < 1e-5);
            world.render_world_snapshot_handle()
        });
        let context = RenderExtractContext::new(
            world_id,
            SceneViewportExtractRequest {
                viewport_size: Some(SIZE),
                settings: ViewportRenderSettings {
                    preview_lighting: false,
                    preview_skybox: false,
                    ..ViewportRenderSettings::default()
                },
                ..SceneViewportExtractRequest::default()
            },
        );
        let extract = self.level.build_render_frame_extract(&context);
        // The input identifies the World. The producer stamps its current generation.
        let generation = self.level.with_world(World::world_generation);
        assert_eq!(extract.world, world_id.with_generation(generation));
        if let Some(changes) = &extract.geometry.scene_changes {
            assert_eq!(changes.world().raw(), world_id.raw());
        }
        assert!(extract.geometry.meshes.is_empty());
        assert_eq!(extract.view.core_pipeline, CorePipelineKind::Core3d);
        assert_eq!(extract.view.camera.transform.translation, camera_position);
        let selected = extract
            .view
            .selected_camera_descriptor()
            .expect("real scene camera");
        assert_eq!(selected.entity, Some(self.camera));
        assert_eq!(selected.camera.transform.translation, camera_position);
        assert_eq!(
            extract.view.selected_camera_volume_layers(),
            &RenderLayerSet::from_scene_schema_v1_mask(LAYER)
        );
        assert_eq!(extract.post_process.volumes.len(), 1);
        extract
    }
}

#[test]
fn level_volume_cpu_resolves_affine_center_and_vignette() {
    let scene = VolumeLevel::new();
    let values = FRAMES.map(|sample| {
        let extract = scene.extract(sample);
        let VolumeShapeExtract::Sphere {
            center,
            radius,
            blend_distance,
        } = &extract.post_process.volumes[0].shape
        else {
            panic!("the actual collider should extract a compact sphere");
        };
        assert!(radius.is_finite() && *radius >= 0.25 && *radius < 1.0);
        assert_eq!(*blend_distance, 0.0);
        let settings = extract
            .post_process
            .resolved_settings_for_camera(
                extract.view.camera.transform.translation,
                extract.view.selected_camera_volume_layers(),
            )
            .unwrap();
        let value = (
            (analytic_center() - *center).length(),
            settings.effect_stack.vignette.intensity,
        );
        println!(
            "POST_VOLUME_AFFINE_CPU label={} error={} vignette={}",
            sample.label(),
            value.0,
            value.1
        );
        value
    });
    assert_eq!(values[0].1, 0.0);
    assert!(values[1].0 < 1e-5 && (values[1].1 - VIGNETTE).abs() < 1e-5);
    assert!(
        values[2].0 < 1e-5 && (values[2].1 - VIGNETTE).abs() < 1e-5,
        "camera at the affine collider origin must resolve the full vignette"
    );
    assert!(values[0].0 < 1e-5);
}

#[test]
fn level_volume_gpu_reads_fresh_affine_vignette_pixels() {
    let scene = VolumeLevel::new();
    let asset_runtime = opened_gpu_project();
    let gpu =
        WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool()).unwrap();
    let viewport = gpu
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .unwrap();
    gpu.set_quality_profile(
        viewport,
        RenderQualityProfile::new("post-volume-affine-origin")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false)
            .with_color_grading(false)
            .with_anti_alias(false),
    )
    .unwrap();
    let mut generation = None;
    let measurements = FRAMES.map(|sample| {
        gpu.submit_frame_extract(viewport, scene.extract(sample)).unwrap();
        let capture = gpu.capture_frame_if_newer(viewport, generation).unwrap().expect("fresh offscreen frame");
        generation = Some(capture.generation);
        let stats = gpu.query_stats().unwrap();
        assert_eq!(stats.last_generation, generation);
        assert_eq!(capture.capture_report.source, RenderCaptureSource::FrameworkOffscreen);
        assert_eq!(UVec2::new(capture.width, capture.height), SIZE);
        assert!(gpu.first_device_fault().is_none());
        assert_post_executed(&stats);
        let report = &stats.last_post_process_effect_stack_report;
        assert!(report.missing_resources.is_empty());
        let diagnostic = stats.device_diagnostics.as_ref().expect("actual device diagnostic");
        let corner = luma(&capture, UVec2::ZERO);
        let center = luma(&capture, UVec2::new(52, 36));
        println!("POST_VOLUME_AFFINE_GPU label={} generation={} backend={} adapter_type={} adapter={:?} corner={} center={} vignette={}",
            sample.label(), capture.generation, stats.capabilities.backend_name, diagnostic.adapter_device_type,
            diagnostic.adapter_name, corner, center, report.active_families.iter().any(|f| f == "vignette"));
        (corner, center, report.active_families.clone())
    });
    let [outside, rigid, shear] = measurements;
    assert!(outside.0 > 40.0, "outside must have visible backdrop");
    assert!(outside.2.is_empty());
    assert!(rigid.0 + 3.0 < outside.0);
    assert_eq!(rigid.2, vec!["vignette".to_string()]);
    assert!((rigid.1 - outside.1).abs() <= 1.0 && (shear.1 - outside.1).abs() <= 1.0);
    // Keep this after the actual readback and rigid/empty controls: a shape
    // assertion must not hide the old wrong final pixels.
    assert!(
        shear.0 + 3.0 < outside.0 && (shear.0 - rigid.0).abs() <= 1.0,
        "sheared affine collider center must darken final-frame corners like the rigid center"
    );
    assert_eq!(shear.2, rigid.2);
}

fn opened_gpu_project() -> ProjectAssetTestRuntime {
    // The production constructor needs a real project for the pipeline cache.
    // This directory only owns test assets/caches; the live scene still comes from Level.
    let output = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .expect("set the native validation output root");
    let output = output
        .canonicalize()
        .expect("physical validation output root");
    #[cfg(windows)]
    {
        let physical = output.to_string_lossy().to_ascii_lowercase();
        let physical = physical.strip_prefix(r"\\?\").unwrap_or(&physical);
        assert!(
            [
                r"d:\cargo-targets",
                r"e:\cargo-targets",
                r"f:\cargo-targets"
            ]
            .iter()
            .any(|allowed| Path::new(physical).starts_with(allowed)),
            "the project and GPU compilation cache must stay in the physical Cargo pool"
        );
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = output.join(format!(
        "post-volume-affine-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create a unique isolated GPU fixture project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let scene_uri = AssetUri::parse("res://scenes/post_volume_affine.scene.toml").unwrap();
    ProjectManifest::new("PostVolumeAffineFixture", scene_uri, 1)
        .save(paths.manifest_path())
        .unwrap();
    let manager = Arc::new(ProjectAssetManager::default());
    manager
        .open_project(root.to_str().unwrap())
        .expect("open the isolated renderer project");
    ProjectAssetTestRuntime::new(manager)
}

fn assert_post_executed(stats: &RenderStats) {
    let index = |executor| {
        stats
            .last_graph_executed_executor_ids
            .iter()
            .position(|id| id == executor)
    };
    let uber = index("post.uber").expect("compiled post.uber executor");
    let transfer = index("post.output-transfer").expect("compiled output-transfer executor");
    assert!(uber < transfer);
    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
}

fn luma(frame: &CapturedFrame, origin: UVec2) -> f32 {
    let mut sum = 0.0;
    for y in origin.y..origin.y + 24 {
        for x in origin.x..origin.x + 24 {
            let offset = ((y * frame.width + x) * 4) as usize;
            sum += 0.2126 * frame.rgba[offset] as f32
                + 0.7152 * frame.rgba[offset + 1] as f32
                + 0.0722 * frame.rgba[offset + 2] as f32;
        }
    }
    sum / (24 * 24) as f32
}
