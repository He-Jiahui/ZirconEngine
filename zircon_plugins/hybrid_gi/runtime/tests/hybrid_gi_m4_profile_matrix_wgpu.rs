use std::{fs, path::PathBuf, sync::Arc};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_plugin_hybrid_gi_runtime::{
    hybrid_gi_runtime_provider_registration, render_feature_descriptor,
    render_pass_executor_registrations, runtime_prepare_collector_registration,
};
use zircon_runtime::asset::assets::{
    texture_asset_from_lightmap_bake_output, AlphaMode, MaterialAsset, ModelAsset,
    ModelPrimitiveAsset,
};
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{AssetReference, AssetUri, MeshVertex, TextureAsset};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, CapturedFrame,
    CorePipelineKind, DisplayMode, EnvironmentExtract, LightProbeGridData, LightmapAtlasDescriptor,
    LightmapAtlasFormat, LightmapAtlasPage, LightmapBakeOutput, LightmapInstanceSlot,
    PreviewEnvironmentExtract, ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiProfile,
    RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState, RenderOverlayExtract,
    RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
    RendererCommon, ShL2Rgb, ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::graphics::WgpuRenderFramework;

const SIZE: UVec2 = UVec2::new(160, 120);
const STATIC_NODE_ID: u64 = 100;
const DYNAMIC_NODE_ID: u64 = 101;
const FORWARD: RenderPipelineHandle = RenderPipelineHandle::new(1);
const DEFERRED: RenderPipelineHandle = RenderPipelineHandle::new(2);
const OUTPUT_PNG: &str = "plan18_hybrid_gi_m4_profile_forward_deferred_wgpu_20260713.png";
const OUTPUT_REPORT: &str = "plan18_hybrid_gi_m4_profile_forward_deferred_wgpu_20260713.txt";
const MOBILITY_OUTPUT_PNG: &str = "plan18_hybrid_gi_m4_mobility_roundtrip_wgpu_20260713.png";
const MOBILITY_OUTPUT_REPORT: &str = "plan18_hybrid_gi_m4_mobility_roundtrip_wgpu_20260713.txt";
const EMISSIVE_OUTPUT_PNG: &str = "plan18_hybrid_gi_m4_moving_emissive_wgpu_20260713.png";
const EMISSIVE_OUTPUT_REPORT: &str = "plan18_hybrid_gi_m4_moving_emissive_wgpu_20260713.txt";

struct Capture {
    frame: CapturedFrame,
    stats: RenderStats,
}

#[test]
fn hybrid_gi_m4_profiles_render_forward_deferred_wgpu_product_matrix() {
    let profiles = [
        ("fully_dynamic", RenderHybridGiProfile::FullyDynamic, false),
        ("indoor_static", RenderHybridGiProfile::IndoorStatic, true),
        ("open_world", RenderHybridGiProfile::OpenWorld, true),
        ("cinematic", RenderHybridGiProfile::Cinematic, true),
    ];
    let mut captures = Vec::with_capacity(8);
    let mut report = String::from(
        "HybridGI M4 Forward+/Deferred profile WGPU product matrix\nrows=forward_plus|deferred\n",
    );
    let mut fully_dynamic_forward = None;
    let mut fully_dynamic_deferred = None;

    for (index, (name, profile, baked)) in profiles.into_iter().enumerate() {
        let forward = capture_profile(FORWARD, profile, baked, 18_400 + index as u64 * 2);
        let deferred = capture_profile(DEFERRED, profile, baked, 18_401 + index as u64 * 2);
        let parity = mean_absolute_rgb_error(&forward.frame, &deferred.frame);
        assert!(visible_pixels(&forward.frame) > 2_000);
        assert!(visible_pixels(&deferred.frame) > 2_000);
        assert_eq!(forward.stats.last_hybrid_gi_graph_executed_pass_count, 4);
        assert_eq!(deferred.stats.last_hybrid_gi_graph_executed_pass_count, 4);
        assert!(forward.stats.last_hybrid_gi_scene_screen_probe_count >= 1);
        assert!(deferred.stats.last_hybrid_gi_scene_screen_probe_count >= 1);
        assert!(forward.stats.last_hybrid_gi_voxel_resident_clipmap_count >= 1);
        assert!(deferred.stats.last_hybrid_gi_voxel_resident_clipmap_count >= 1);
        assert!(
            parity < 8.0,
            "{name} Forward+/Deferred output diverged: MAE={parity:.4}"
        );
        report.push_str(&format!(
            "{name}_forward_center={:?}\n{name}_deferred_center={:?}\n{name}_parity_mae={parity:.6}\n{name}_forward_passes={}\n{name}_deferred_passes={}\n{name}_forward_probe_tiles={}\n{name}_deferred_probe_tiles={}\n{name}_forward_voxel_clipmaps={}\n{name}_deferred_voxel_clipmaps={}\n",
            center_rgba(&forward.frame),
            center_rgba(&deferred.frame),
            forward.stats.last_hybrid_gi_graph_executed_pass_count,
            deferred.stats.last_hybrid_gi_graph_executed_pass_count,
            forward.stats.last_hybrid_gi_probe_trace_tile_count,
            deferred.stats.last_hybrid_gi_probe_trace_tile_count,
            forward.stats.last_hybrid_gi_voxel_resident_clipmap_count,
            deferred.stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ));
        report.push_str(&format!(
            "{name}_forward_screen_probes={}\n{name}_deferred_screen_probes={}\n",
            forward.stats.last_hybrid_gi_scene_screen_probe_count,
            deferred.stats.last_hybrid_gi_scene_screen_probe_count,
        ));
        if profile == RenderHybridGiProfile::FullyDynamic {
            fully_dynamic_forward = Some(forward.frame.clone());
            fully_dynamic_deferred = Some(deferred.frame.clone());
        } else {
            let forward_baked_delta = mean_absolute_rgb_error(
                fully_dynamic_forward
                    .as_ref()
                    .expect("FullyDynamic must be captured first"),
                &forward.frame,
            );
            let deferred_baked_delta = mean_absolute_rgb_error(
                fully_dynamic_deferred
                    .as_ref()
                    .expect("FullyDynamic must be captured first"),
                &deferred.frame,
            );
            assert!(forward_baked_delta > 1.0);
            assert!(deferred_baked_delta > 1.0);
            report.push_str(&format!(
                "{name}_forward_baked_delta_mae={forward_baked_delta:.6}\n{name}_deferred_baked_delta_mae={deferred_baked_delta:.6}\n"
            ));
        }
        captures.push(forward.frame);
        captures.push(deferred.frame);
    }

    let output_dir = output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_matrix_png(output_dir.join(OUTPUT_PNG), &captures);
    report.push_str(&format!(
        "png={OUTPUT_PNG}\nwidth={}\nheight={}\nwgpu_product=true\n",
        SIZE.x * 4 + 6,
        SIZE.y * 2 + 2,
    ));
    fs::write(output_dir.join(OUTPUT_REPORT), report).unwrap();
}

#[test]
fn hybrid_gi_m4_mobility_round_trip_releases_and_restores_static_lightmap() {
    let mut frames = Vec::with_capacity(6);
    let mut report =
        String::from("rows=forward_plus|deferred\ncolumns=static|dynamic|static_restored\n");
    for (pipeline_name, pipeline) in [("forward_plus", FORWARD), ("deferred", DEFERRED)] {
        let assets = Arc::new(ProjectAssetManager::default());
        let model = register_plane(&assets);
        let material = register_material(&assets);
        let environment = register_environment(&assets, true);
        let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
            assets,
            [render_feature_descriptor()],
            render_pass_executor_registrations(),
            [runtime_prepare_collector_registration()],
            [hybrid_gi_runtime_provider_registration()],
            Vec::new(),
        )
        .unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(SIZE))
            .unwrap();
        framework.set_pipeline_asset(viewport, pipeline).unwrap();
        framework
            .set_quality_profile(viewport, quality_profile(true))
            .unwrap();

        let static_extract = scene_extract_with_primary_mobility(
            18_500,
            RenderHybridGiProfile::IndoorStatic,
            environment.clone(),
            model,
            material,
            Mobility::Static,
        );
        let dynamic_extract = scene_extract_with_primary_mobility(
            18_500,
            RenderHybridGiProfile::IndoorStatic,
            environment,
            model,
            material,
            Mobility::Dynamic,
        );
        let static_frame = submit_and_capture(&framework, viewport, &static_extract, 3);
        let dynamic_frame = submit_and_capture(&framework, viewport, &dynamic_extract, 2);
        let restored_frame = submit_and_capture(&framework, viewport, &static_extract, 2);
        let released_delta = mean_absolute_rgb_error(&static_frame, &dynamic_frame);
        let restored_delta = mean_absolute_rgb_error(&static_frame, &restored_frame);
        assert!(
            released_delta > 1.0,
            "{pipeline_name} dynamic mobility must release the static lightmap: {released_delta:.6}"
        );
        assert!(
            restored_delta < released_delta * 0.25,
            "{pipeline_name} static restore retained stale energy: released={released_delta:.6}, restored={restored_delta:.6}"
        );
        report.push_str(&format!(
            "{pipeline_name}_released_mae={released_delta:.6}\n{pipeline_name}_restored_mae={restored_delta:.6}\n{pipeline_name}_static_center={:?}\n{pipeline_name}_dynamic_center={:?}\n{pipeline_name}_restored_center={:?}\n",
            center_rgba(&static_frame),
            center_rgba(&dynamic_frame),
            center_rgba(&restored_frame),
        ));
        frames.extend([static_frame, dynamic_frame, restored_frame]);
        framework.destroy_viewport(viewport).unwrap();
    }
    let output_dir = output_dir();
    write_mobility_png(output_dir.join(MOBILITY_OUTPUT_PNG), &frames);
    fs::write(output_dir.join(MOBILITY_OUTPUT_REPORT), report).unwrap();
}

#[test]
fn hybrid_gi_m4_moving_emissive_invalidates_and_restores_without_ghosting() {
    let mut frames = Vec::with_capacity(6);
    let mut report = String::from("rows=forward_plus|deferred\ncolumns=origin|moved|restored\n");
    for (pipeline_name, pipeline) in [("forward_plus", FORWARD), ("deferred", DEFERRED)] {
        let assets = Arc::new(ProjectAssetManager::default());
        let model = register_plane(&assets);
        let material = register_material(&assets);
        let emissive = register_emissive_material(&assets);
        let environment = register_environment(&assets, true);
        let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
            assets,
            [render_feature_descriptor()],
            render_pass_executor_registrations(),
            [runtime_prepare_collector_registration()],
            [hybrid_gi_runtime_provider_registration()],
            Vec::new(),
        )
        .unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(SIZE))
            .unwrap();
        framework.set_pipeline_asset(viewport, pipeline).unwrap();
        framework
            .set_quality_profile(viewport, quality_profile(true))
            .unwrap();

        let origin = emissive_scene_extract(
            18_600,
            environment.clone(),
            model,
            material,
            emissive,
            Vec3::new(1.6, 0.0, 0.0),
        );
        let moved = emissive_scene_extract(
            18_600,
            environment,
            model,
            material,
            emissive,
            Vec3::new(0.55, 0.55, 0.0),
        );
        let origin_frame = submit_and_capture(&framework, viewport, &origin, 3);
        let moved_frame = submit_and_capture(&framework, viewport, &moved, 2);
        let restored_frame = submit_and_capture(&framework, viewport, &origin, 2);
        let moved_delta = mean_absolute_rgb_error(&origin_frame, &moved_frame);
        let restored_delta = mean_absolute_rgb_error(&origin_frame, &restored_frame);
        assert!(moved_delta > 1.0);
        assert!(restored_delta < moved_delta * 0.25);
        report.push_str(&format!(
            "{pipeline_name}_moved_mae={moved_delta:.6}\n{pipeline_name}_restored_mae={restored_delta:.6}\n"
        ));
        frames.extend([origin_frame, moved_frame, restored_frame]);
        framework.destroy_viewport(viewport).unwrap();
    }
    let output_dir = output_dir();
    write_mobility_png(output_dir.join(EMISSIVE_OUTPUT_PNG), &frames);
    fs::write(output_dir.join(EMISSIVE_OUTPUT_REPORT), report).unwrap();
}

fn capture_profile(
    pipeline: RenderPipelineHandle,
    profile: RenderHybridGiProfile,
    baked: bool,
    world_id: u64,
) -> Capture {
    let assets = Arc::new(ProjectAssetManager::default());
    let model = register_plane(&assets);
    let material = register_material(&assets);
    let environment = register_environment(&assets, baked);
    let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
        assets,
        [render_feature_descriptor()],
        render_pass_executor_registrations(),
        [runtime_prepare_collector_registration()],
        [hybrid_gi_runtime_provider_registration()],
        Vec::new(),
    )
    .expect("HybridGI WGPU framework should initialize");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .expect("profile matrix viewport should be created");
    framework.set_pipeline_asset(viewport, pipeline).unwrap();
    framework
        .set_quality_profile(viewport, quality_profile(baked))
        .unwrap();
    let extract = scene_extract(world_id, profile, environment, model, material);
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    framework.submit_frame_extract(viewport, extract).unwrap();
    let stats = framework.query_stats().unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("profile matrix frame should be capturable");
    framework.destroy_viewport(viewport).unwrap();
    Capture { frame, stats }
}

fn scene_extract(
    world_id: u64,
    profile: RenderHybridGiProfile,
    environment: EnvironmentExtract,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    scene_extract_with_primary_mobility(
        world_id,
        profile,
        environment,
        model,
        material,
        Mobility::Static,
    )
}

fn scene_extract_with_primary_mobility(
    world_id: u64,
    profile: RenderHybridGiProfile,
    environment: EnvironmentExtract,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    primary_mobility: Mobility,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 7.0), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 48.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 30.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                fixture_mesh(
                    STATIC_NODE_ID,
                    Vec3::new(-1.55, 0.0, 0.0),
                    Vec3::new(1.35, 1.65, 1.0),
                    model,
                    material,
                    primary_mobility,
                ),
                fixture_mesh(
                    DYNAMIC_NODE_ID,
                    Vec3::new(1.6, 0.0, 0.0),
                    Vec3::splat(1.25),
                    ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
                    material,
                    Mobility::Dynamic,
                ),
            ],
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 900,
                light_id: 900,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                direction: Vec3::new(-0.35, -0.65, -1.0).normalize_or_zero(),
                color: Vec3::new(1.0, 0.55, 0.3),
                intensity: 2.5,
                mobility: Mobility::Dynamic,
                shadow: None,
            }],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            false,
            Vec4::new(0.005, 0.007, 0.01, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(world_id), snapshot)
            .with_viewport_size(SIZE);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        profile,
        debug_view: RenderHybridGiDebugView::None,
        ..RenderHybridGiExtract::default()
    });
    extract
}

fn emissive_scene_extract(
    world_id: u64,
    environment: EnvironmentExtract,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    emissive: ResourceHandle<MaterialMarker>,
    emissive_translation: Vec3,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 7.0), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 48.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 30.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![
                fixture_mesh(
                    STATIC_NODE_ID,
                    Vec3::new(-1.55, 0.0, 0.0),
                    Vec3::new(1.35, 1.65, 1.0),
                    model,
                    material,
                    Mobility::Static,
                ),
                fixture_mesh(
                    DYNAMIC_NODE_ID,
                    emissive_translation,
                    Vec3::splat(1.25),
                    ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
                    emissive,
                    Mobility::Dynamic,
                ),
            ],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            false,
            Vec4::new(0.005, 0.007, 0.01, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(world_id), snapshot)
            .with_viewport_size(SIZE);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        profile: RenderHybridGiProfile::IndoorStatic,
        debug_view: RenderHybridGiDebugView::None,
        ..RenderHybridGiExtract::default()
    });
    extract
}

fn fixture_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    mobility: Mobility,
) -> RenderMeshSnapshot {
    let transform = Transform {
        translation,
        scale,
        ..Transform::default()
    };
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility,
        static_state: RenderMeshStaticState::new(mobility == Mobility::Static, 1, 1),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: mobility == Mobility::Static,
            ..RendererCommon::default()
        },
    }
}

fn register_environment(assets: &ProjectAssetManager, enabled: bool) -> EnvironmentExtract {
    if !enabled {
        return EnvironmentExtract::disabled();
    }
    let output = baked_output();
    let uri = AssetUri::parse("res://lighting/m4-profile.lightmap-array").unwrap();
    let atlas_id = ResourceId::from_locator(&uri);
    let texture = texture_asset_from_lightmap_bake_output(uri.clone(), &output).unwrap();
    assets
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(atlas_id, ResourceKind::Texture, uri)
                .with_source_hash("m4-profile-lightmap-v1"),
            texture,
        )
        .unwrap();
    let (contract, probes) = output.into_consume_contract(atlas_id).unwrap();
    EnvironmentExtract::disabled()
        .try_with_baked_lighting(contract, probes)
        .unwrap()
}

fn baked_output() -> LightmapBakeOutput {
    let mut sh = ShL2Rgb::ZERO;
    sh.0[0] = Vec3::new(1.063_472_3, 1.772_453_9, 2.481_435_3);
    LightmapBakeOutput {
        contract_version: 1,
        request_id: 1_103,
        scene_revision: 7,
        light_set_generation: 3,
        atlas: LightmapAtlasDescriptor {
            page_size: 2,
            page_count: 1,
            format: LightmapAtlasFormat::Rgba16Float,
        },
        atlas_pages: vec![LightmapAtlasPage {
            page_index: 0,
            texels_rgba16f_le: vec![
                0, 60, 205, 56, 205, 52, 0, 60, 31, 45, 31, 45, 102, 46, 0, 60, 31, 45, 31, 45,
                102, 46, 0, 60, 205, 52, 0, 56, 0, 60, 0, 60,
            ],
        }],
        slots: vec![(
            render_mesh_stable_instance_key(STATIC_NODE_ID, 0),
            LightmapInstanceSlot {
                atlas_page: 0,
                uv_rect: Vec4::new(1.0, 1.0, 0.0, 0.0),
            },
        )],
        probe_grid: Some(LightProbeGridData {
            light_set_generation: 3,
            bounds_min: Vec3::new(-4.0, -3.0, -3.0),
            cell_size: Vec3::new(8.0, 6.0, 6.0),
            dims: [2, 2, 2],
            sh: vec![sh; 8],
        }),
    }
}

fn register_plane(assets: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse("res://models/m4-profile-plane.zmodel").unwrap();
    let id = ResourceId::from_locator(&uri);
    let vertices = vec![
        MeshVertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::Z, Vec2::new(0.0, 1.0))
            .with_uv1(Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(1.0, -1.0, 0.0), Vec3::Z, Vec2::new(1.0, 1.0))
            .with_uv1(Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(-1.0, 1.0, 0.0), Vec3::Z, Vec2::new(0.0, 0.0))
            .with_uv1(Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(1.0, 1.0, 0.0), Vec3::Z, Vec2::new(1.0, 0.0))
            .with_uv1(Vec2::new(1.0, 0.0)),
    ];
    assets
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Model, uri.clone())
                .with_source_hash("m4-profile-plane-v1"),
            ModelAsset {
                uri,
                primitives: vec![ModelPrimitiveAsset {
                    vertices,
                    indices: vec![0, 1, 2, 2, 1, 3],
                    mesh: None,
                    virtual_geometry: None,
                }],
            },
        )
        .unwrap();
    ResourceHandle::new(id)
}

fn register_material(assets: &ProjectAssetManager) -> ResourceHandle<MaterialMarker> {
    register_material_with_emissive(
        assets,
        "res://materials/m4-profile-white.zmaterial",
        [0.0; 3],
    )
}

fn register_emissive_material(assets: &ProjectAssetManager) -> ResourceHandle<MaterialMarker> {
    register_material_with_emissive(
        assets,
        "res://materials/m4-profile-emissive.zmaterial",
        [3.0, 0.35, 0.08],
    )
}

fn register_material_with_emissive(
    assets: &ProjectAssetManager,
    uri: &str,
    emissive: [f32; 3],
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    let id = ResourceId::from_locator(&uri);
    assets
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri)
                .with_source_hash("m4-profile-white-v1"),
            MaterialAsset {
                name: Some("M4 Profile Material".to_string()),
                shader: AssetReference::from_locator(
                    AssetUri::parse("builtin://shader/pbr.wgsl").unwrap(),
                ),
                parent: None,
                options: Default::default(),
                queue: None,
                base_color: [0.92, 0.92, 0.92, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 0.72,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive,
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: true,
                property_values: Default::default(),
                texture_slots: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .unwrap();
    ResourceHandle::new(id)
}

fn quality_profile(baked: bool) -> RenderQualityProfile {
    RenderQualityProfile::new("m4-profile-matrix")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(true)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(false)
        .with_baked_lighting(baked)
        .with_hybrid_global_illumination(true)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
        .with_async_compute(false)
}

fn mean_absolute_rgb_error(left: &CapturedFrame, right: &CapturedFrame) -> f64 {
    assert_eq!((left.width, left.height), (right.width, right.height));
    let sum = left
        .rgba
        .chunks_exact(4)
        .zip(right.rgba.chunks_exact(4))
        .map(|(a, b)| {
            u64::from(a[0].abs_diff(b[0]))
                + u64::from(a[1].abs_diff(b[1]))
                + u64::from(a[2].abs_diff(b[2]))
        })
        .sum::<u64>();
    sum as f64 / (u64::from(left.width) * u64::from(left.height) * 3) as f64
}

fn visible_pixels(frame: &CapturedFrame) -> usize {
    frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[3] > 0 && pixel[..3].iter().any(|channel| *channel > 2))
        .count()
}

fn center_rgba(frame: &CapturedFrame) -> [u8; 4] {
    let index = ((frame.height / 2 * frame.width + frame.width / 2) * 4) as usize;
    frame.rgba[index..index + 4].try_into().unwrap()
}

fn submit_and_capture(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
    frame_count: usize,
) -> CapturedFrame {
    for _ in 0..frame_count {
        framework
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
    }
    framework
        .capture_frame(viewport)
        .unwrap()
        .expect("mobility frame should be capturable")
}

fn write_matrix_png(path: PathBuf, captures: &[CapturedFrame]) {
    const GAP: u32 = 2;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(
        SIZE.x * 4 + GAP * 3,
        SIZE.y * 2 + GAP,
        |x, y| {
            let column = x / (SIZE.x + GAP);
            let row = y / (SIZE.y + GAP);
            let local_x = x % (SIZE.x + GAP);
            let local_y = y % (SIZE.y + GAP);
            if column >= 4 || row >= 2 || local_x >= SIZE.x || local_y >= SIZE.y {
                return Rgba([235, 239, 245, 255]);
            }
            let capture_index = column as usize * 2 + row as usize;
            let frame = &captures[capture_index];
            let index = ((local_y * frame.width + local_x) * 4) as usize;
            Rgba(frame.rgba[index..index + 4].try_into().unwrap())
        },
    );
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn write_mobility_png(path: PathBuf, captures: &[CapturedFrame]) {
    const GAP: u32 = 2;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(
        SIZE.x * 3 + GAP * 2,
        SIZE.y * 2 + GAP,
        |x, y| {
            let column = x / (SIZE.x + GAP);
            let row = y / (SIZE.y + GAP);
            let local_x = x % (SIZE.x + GAP);
            let local_y = y % (SIZE.y + GAP);
            if column >= 3 || row >= 2 || local_x >= SIZE.x || local_y >= SIZE.y {
                return Rgba([235, 239, 245, 255]);
            }
            let frame = &captures[row as usize * 3 + column as usize];
            let index = ((local_y * frame.width + local_x) * 4) as usize;
            Rgba(frame.rgba[index..index + 4].try_into().unwrap())
        },
    );
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/tests/runtime/render")
}
