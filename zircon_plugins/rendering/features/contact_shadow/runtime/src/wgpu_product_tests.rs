use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use zircon_runtime::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderAmbientLightSnapshot,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    RendererCommon, ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
    TaskPool,
};
use zircon_runtime::graphics::WgpuRenderFramework;

use super::{
    render_feature_descriptor, render_pass_executor_registration, CONTACT_SHADOW_PIPELINE_LABEL,
    EXECUTOR_ID, FEATURE_NAME, PASS_NAME,
};

const TEST_ASSET_MODULE_NAME: &str = "ContactShadowProductAssetRuntime";
const TEST_ASSET_SERVICE_NAME: &str =
    "ContactShadowProductAssetRuntime.Manager.ProjectAssetManager";

struct ProjectAssetTestRuntime {
    runtime: CoreRuntime,
    access: ProjectAssetManagerAccess,
}

impl ProjectAssetTestRuntime {
    fn new(manager: Arc<ProjectAssetManager>) -> Self {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new(TEST_ASSET_MODULE_NAME, "contact shadow product assets")
                    .with_manager(ManagerDescriptor::new(
                        RegistryName::from_parts(
                            TEST_ASSET_MODULE_NAME,
                            ServiceKind::Manager,
                            "ProjectAssetManager",
                        ),
                        StartupMode::Immediate,
                        Vec::new(),
                        Arc::new(move |_| {
                            Ok(
                                Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                    as ServiceObject,
                            )
                        }),
                    )),
            )
            .expect("contact shadow ProjectAssetManager service should register");
        runtime
            .activate_module(TEST_ASSET_MODULE_NAME)
            .expect("contact shadow ProjectAssetManager module should activate");
        let core = runtime.handle();
        let handle = manager_service_handle(&core, TEST_ASSET_SERVICE_NAME)
            .expect("contact shadow ProjectAssetManager handle should resolve");
        Self {
            runtime,
            access: ProjectAssetManagerAccess::new(core, handle),
        }
    }

    fn access(&self) -> ProjectAssetManagerAccess {
        self.access.clone()
    }

    fn worker_pool(&self) -> TaskPool {
        self.runtime.task_graph().worker_pool().clone()
    }
}

#[test]
fn contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_contact_shadow_material(
        asset_manager.as_ref(),
        "res://materials/contact_shadow_receiver.zmaterial",
        "ContactShadowReceiver",
        [0.72, 0.70, 0.64, 1.0],
    );
    let blocker_material = register_contact_shadow_material(
        asset_manager.as_ref(),
        "res://materials/contact_shadow_blocker.zmaterial",
        "ContactShadowBlocker",
        [0.34, 0.34, 0.32, 1.0],
    );
    let asset_runtime = ProjectAssetTestRuntime::new(Arc::clone(&asset_manager));

    let contact_shadow_framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [render_feature_descriptor()],
        [render_pass_executor_registration()],
        Vec::new(),
        asset_runtime.worker_pool(),
    )
    .expect("contact shadow pluginized WGPU framework");
    let baseline_framework =
        WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool())
            .expect("baseline WGPU framework");

    let (contact_frame, contact_stats) = render_contact_shadow_frame(
        &contact_shadow_framework,
        viewport_size,
        "contact-shadow-enabled",
        contact_shadow_scene_extract(viewport_size, receiver_material, blocker_material),
    );
    let (baseline_frame, baseline_stats) = render_contact_shadow_frame(
        &baseline_framework,
        viewport_size,
        "contact-shadow-disabled",
        contact_shadow_scene_extract(viewport_size, receiver_material, blocker_material),
    );

    assert_contact_shadow_wgpu_stats(&contact_stats);
    assert!(
        !baseline_stats
            .last_graph_executed_executor_ids
            .contains(&EXECUTOR_ID.to_string()),
        "baseline graph should not execute contact shadow; executors={:?}",
        baseline_stats.last_graph_executed_executor_ids
    );

    let profile = frame_contact_shadow_darkening_profile(&baseline_frame, &contact_frame);
    let frame_delta = frame_rgb_abs_delta(&baseline_frame, &contact_frame);
    assert!(
        profile.darkened_pixels > 160 && profile.luma_delta > 1_200.0 && frame_delta > 8_000,
        "contact shadow pass should visibly darken screen-space contact pixels; darkened_pixels={} luma_delta={:.2} frame_delta={frame_delta}",
        profile.darkened_pixels,
        profile.luma_delta
    );
}

#[test]
fn contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions() {
    let viewport_size = UVec2::new(192, 128);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_contact_shadow_material(
        asset_manager.as_ref(),
        "res://materials/contact_shadow_wide_receiver.zmaterial",
        "ContactShadowWideReceiver",
        [0.74, 0.72, 0.66, 1.0],
    );
    let blocker_material = register_contact_shadow_material(
        asset_manager.as_ref(),
        "res://materials/contact_shadow_wide_blocker.zmaterial",
        "ContactShadowWideBlocker",
        [0.31, 0.32, 0.30, 1.0],
    );
    let asset_runtime = ProjectAssetTestRuntime::new(Arc::clone(&asset_manager));

    let contact_shadow_framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_runtime.access(),
        [render_feature_descriptor()],
        [render_pass_executor_registration()],
        Vec::new(),
        asset_runtime.worker_pool(),
    )
    .expect("contact shadow pluginized WGPU framework");
    let baseline_framework =
        WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool())
            .expect("baseline WGPU framework");

    let (contact_frame, contact_stats) = render_contact_shadow_frame(
        &contact_shadow_framework,
        viewport_size,
        "contact-shadow-wide-enabled",
        wide_contact_shadow_scene_extract(viewport_size, receiver_material, blocker_material),
    );
    let (baseline_frame, baseline_stats) = render_contact_shadow_frame(
        &baseline_framework,
        viewport_size,
        "contact-shadow-wide-disabled",
        wide_contact_shadow_scene_extract(viewport_size, receiver_material, blocker_material),
    );

    assert_contact_shadow_wgpu_stats(&contact_stats);
    assert!(
        !baseline_stats
            .last_graph_executed_executor_ids
            .contains(&EXECUTOR_ID.to_string()),
        "baseline graph should not execute contact shadow; executors={:?}",
        baseline_stats.last_graph_executed_executor_ids
    );

    let whole_frame = frame_contact_shadow_darkening_profile(&baseline_frame, &contact_frame);
    assert!(
        whole_frame.darkened_pixels > 300 && whole_frame.luma_delta > 2_400.0,
        "wide contact shadow pass should darken multiple contact regions; whole_frame={whole_frame:?}"
    );

    let left_contact = frame_contact_shadow_darkening_profile_in_rect(
        &baseline_frame,
        &contact_frame,
        FrameRect::new(26, 58, 66, 114),
    );
    let center_contact = frame_contact_shadow_darkening_profile_in_rect(
        &baseline_frame,
        &contact_frame,
        FrameRect::new(76, 50, 116, 112),
    );
    let right_contact = frame_contact_shadow_darkening_profile_in_rect(
        &baseline_frame,
        &contact_frame,
        FrameRect::new(126, 58, 166, 114),
    );
    for (label, profile) in [
        ("left", left_contact),
        ("center", center_contact),
        ("right", right_contact),
    ] {
        assert!(
            profile.darkened_pixels > 28 && profile.luma_delta > 180.0,
            "{label} contact region should receive localized darkening; profile={profile:?}"
        );
    }

    let open_receiver = frame_contact_shadow_darkening_profile_in_rect(
        &baseline_frame,
        &contact_frame,
        FrameRect::new(84, 22, 108, 44),
    );
    assert!(
        open_receiver.darkened_pixels < whole_frame.darkened_pixels / 5,
        "open receiver region should not account for most contact shadow darkening; open={open_receiver:?} whole={whole_frame:?}"
    );
}

fn register_contact_shadow_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    name: &str,
    base_color: [f32; 4],
) -> ResourceId {
    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::new(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("contact shadow material insert");
    material_id
}

fn render_contact_shadow_frame(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
    frame_extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, contact_shadow_quality_profile(profile_name))
        .unwrap();
    framework
        .submit_frame_extract(viewport, frame_extract)
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("contact shadow product frame should be capturable");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn contact_shadow_quality_profile(name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(true)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn contact_shadow_scene_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    blocker_material: ResourceId,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(61_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.05, 2.15),
                        Vec3::new(0.0, 0.0, 0.16),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    contact_shadow_mesh(
                        61_100,
                        Transform {
                            scale: Vec3::new(3.2, 2.2, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    contact_shadow_mesh(
                        61_101,
                        Transform {
                            translation: Vec3::new(-0.12, -0.04, 0.50),
                            scale: Vec3::new(0.38, 0.24, 0.96),
                            ..Transform::default()
                        },
                        blocker_material,
                    ),
                ],
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 61_200,
                    light_id: 61_200,
                    layer_mask: default_render_layer_set(),
                    direction: Vec3::new(0.30, 0.36, -1.0).normalize(),
                    color: Vec3::ONE,
                    intensity: 1.1,
                    mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
                    shadow: None,
                }],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: vec![RenderAmbientLightSnapshot {
                    color: Vec3::ONE,
                    intensity: 0.18,
                    renderer_degraded: false,
                    degradation_reason: None,
                }],
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size)
}

fn wide_contact_shadow_scene_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    blocker_material: ResourceId,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(62_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.45, 2.35),
                        Vec3::new(0.0, 0.0, 0.18),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    contact_shadow_mesh(
                        62_100,
                        Transform {
                            scale: Vec3::new(4.4, 2.65, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    contact_shadow_mesh(
                        62_101,
                        Transform {
                            translation: Vec3::new(-0.92, -0.06, 0.44),
                            scale: Vec3::new(0.28, 0.22, 0.84),
                            ..Transform::default()
                        },
                        blocker_material,
                    ),
                    contact_shadow_mesh(
                        62_102,
                        Transform {
                            translation: Vec3::new(0.0, 0.14, 0.55),
                            scale: Vec3::new(0.34, 0.28, 1.06),
                            ..Transform::default()
                        },
                        blocker_material,
                    ),
                    contact_shadow_mesh(
                        62_103,
                        Transform {
                            translation: Vec3::new(0.92, -0.03, 0.47),
                            scale: Vec3::new(0.27, 0.25, 0.90),
                            ..Transform::default()
                        },
                        blocker_material,
                    ),
                ],
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 62_200,
                    light_id: 62_200,
                    layer_mask: default_render_layer_set(),
                    direction: Vec3::new(0.25, 0.42, -1.0).normalize(),
                    color: Vec3::ONE,
                    intensity: 1.15,
                    mobility: zircon_runtime::core::framework::scene::Mobility::Dynamic,
                    shadow: None,
                }],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: vec![RenderAmbientLightSnapshot {
                    color: Vec3::ONE,
                    intensity: 0.18,
                    renderer_degraded: false,
                    degradation_reason: None,
                }],
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size)
}

fn contact_shadow_mesh(
    node_id: u64,
    transform: Transform,
    material: ResourceId,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: RendererCommon {
            layer_mask: default_render_layer_set(),
            is_static: false,
            ..RendererCommon::default()
        },
    }
}

fn default_render_layer_set() -> RenderLayerSet {
    RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)
}

fn assert_contact_shadow_wgpu_stats(stats: &RenderStats) {
    assert!(
        stats
            .last_effective_features
            .contains(&FEATURE_NAME.to_string()),
        "contact shadow feature should be enabled; features={:?}",
        stats.last_effective_features
    );
    assert!(
        stats
            .last_graph_executed_passes
            .contains(&PASS_NAME.to_string()),
        "contact shadow graph pass should execute; passes={:?}",
        stats.last_graph_executed_passes
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&EXECUTOR_ID.to_string()),
        "contact shadow executor should run; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(
        stats.last_graph_compute_dispatch_count > 0
            && stats.last_graph_compute_matched_workload_count > 0,
        "contact shadow should record matched compute workload `{CONTACT_SHADOW_PIPELINE_LABEL}`; dispatch_count={} matched_workloads={}",
        stats.last_graph_compute_dispatch_count,
        stats.last_graph_compute_matched_workload_count
    );
    assert_eq!(stats.last_graph_compute_missing_dispatch_count, 0);
    assert_eq!(stats.last_graph_compute_workload_mismatch_count, 0);
    assert_eq!(stats.last_graph_compute_unexpected_dispatch_count, 0);
}

#[derive(Clone, Copy, Debug)]
struct ContactShadowDarkeningProfile {
    darkened_pixels: usize,
    luma_delta: f32,
}

#[derive(Clone, Copy, Debug)]
struct FrameRect {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl FrameRect {
    fn new(min_x: u32, min_y: u32, max_x: u32, max_y: u32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

fn frame_contact_shadow_darkening_profile(
    baseline: &CapturedFrame,
    contact_shadow: &CapturedFrame,
) -> ContactShadowDarkeningProfile {
    assert_eq!(
        (baseline.width, baseline.height),
        (contact_shadow.width, contact_shadow.height)
    );
    let mut darkened_pixels = 0;
    let mut luma_delta = 0.0;
    for (baseline_pixel, contact_pixel) in baseline
        .rgba
        .chunks_exact(4)
        .zip(contact_shadow.rgba.chunks_exact(4))
    {
        let baseline_luma = rgb_luma(baseline_pixel);
        let contact_luma = rgb_luma(contact_pixel);
        if baseline_luma > contact_luma + 1.5 {
            darkened_pixels += 1;
            luma_delta += baseline_luma - contact_luma;
        }
    }

    ContactShadowDarkeningProfile {
        darkened_pixels,
        luma_delta,
    }
}

fn frame_contact_shadow_darkening_profile_in_rect(
    baseline: &CapturedFrame,
    contact_shadow: &CapturedFrame,
    rect: FrameRect,
) -> ContactShadowDarkeningProfile {
    assert_eq!(
        (baseline.width, baseline.height),
        (contact_shadow.width, contact_shadow.height)
    );
    assert!(rect.min_x < rect.max_x && rect.min_y < rect.max_y);
    assert!(rect.max_x <= baseline.width && rect.max_y <= baseline.height);

    let mut darkened_pixels = 0;
    let mut luma_delta = 0.0;
    for y in rect.min_y..rect.max_y {
        for x in rect.min_x..rect.max_x {
            let offset = ((y * baseline.width + x) * 4) as usize;
            let baseline_pixel = &baseline.rgba[offset..offset + 4];
            let contact_pixel = &contact_shadow.rgba[offset..offset + 4];
            let baseline_luma = rgb_luma(baseline_pixel);
            let contact_luma = rgb_luma(contact_pixel);
            if baseline_luma > contact_luma + 1.5 {
                darkened_pixels += 1;
                luma_delta += baseline_luma - contact_luma;
            }
        }
    }

    ContactShadowDarkeningProfile {
        darkened_pixels,
        luma_delta,
    }
}

fn frame_rgb_abs_delta(a: &CapturedFrame, b: &CapturedFrame) -> u64 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    a.rgba
        .chunks_exact(4)
        .zip(b.rgba.chunks_exact(4))
        .map(|(lhs, rhs)| {
            lhs[..3]
                .iter()
                .zip(&rhs[..3])
                .map(|(x, y)| (*x as i16 - *y as i16).unsigned_abs() as u64)
                .sum::<u64>()
        })
        .sum()
}

fn rgb_luma(pixel: &[u8]) -> f32 {
    pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722
}
