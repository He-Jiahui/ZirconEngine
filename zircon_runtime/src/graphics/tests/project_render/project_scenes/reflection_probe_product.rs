use std::sync::Arc;

use crate::asset::{
    texture_asset_from_ibl_bake_artifact_pmrem, AssetManager, ProjectAssetManager, TextureAsset,
};
use crate::core::framework::render::{
    build_source_cubemap_from_equirect, CapturedFrame, CorePipelineKind, EnvironmentExtract,
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, ProbeInfluenceShape, ProceduralSkyParams, ProjectionMode,
    ReflectionProbeData, RenderFrameExtract, RenderFramework, RenderOverlayExtract,
    RenderQualityProfile, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
    ViewportRenderSettings,
};
use crate::core::math::{Quat, UVec2, Vec3};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
use crate::graphics::{RenderFeatureDescriptor, SceneRenderer, ViewportFrame, WgpuRenderFramework};
use crate::scene::world::World;

use super::*;

const PROBE_PRODUCT_OUTPUT_SIZE: UVec2 = UVec2::new(960, 540);
const PROBE_PRODUCT_ORTHO_SIZE: f32 = 4.0;
const PROBE_PLANE_HALF_WIDTH: f32 = 3.0;
const PROBE_PLANE_HALF_HEIGHT: f32 = 1.25;
const LEFT_PROBE_PMREM_COLOR: [f32; 4] = [0.9, 0.01, 0.01, 1.0];
const RIGHT_PROBE_PMREM_COLOR: [f32; 4] = [0.01, 0.01, 0.9, 1.0];

#[test]
fn render_product_probe_blend_boundary_smooth() {
    let fixture = ProbeProductFixture::new("probe_blend_boundary");
    let frame = fixture.render_direct(fixture.environment_with_probes());

    assert_probe_boundary_is_smooth(frame.width, frame.height, &frame.rgba);

    fixture.cleanup();
}

#[test]
fn render_product_probe_feature_off_falls_back_to_skybox() {
    let fixture = ProbeProductFixture::new("probe_feature_off");
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::clone(&fixture.asset_manager),
        [reflection_probe_render_feature_descriptor()],
        Vec::new(),
        Vec::new(),
    )
    .expect("probe product render framework");
    let viewport = framework
        .create_viewport(
            crate::core::framework::render::RenderViewportDescriptor::new(
                PROBE_PRODUCT_OUTPUT_SIZE,
            ),
        )
        .expect("probe product viewport");

    let enabled = fixture.capture_with_framework(&framework, viewport, true, true, 1);
    let disabled = fixture.capture_with_framework(&framework, viewport, false, true, 2);
    let sky_only = fixture.capture_with_framework(&framework, viewport, true, false, 3);
    let fallback_error = mean_absolute_rgb_error(&disabled.rgba, &sky_only.rgba);
    let probe_difference = mean_absolute_rgb_error(&enabled.rgba, &sky_only.rgba);

    assert!(
        fallback_error <= 0.25,
        "feature-off probe frame must match the no-probe sky fallback, MAE={fallback_error}"
    );
    assert!(
        probe_difference >= 4.0,
        "enabled probes must visibly differ from sky fallback, MAE={probe_difference}"
    );

    fixture.cleanup();
}

#[test]
#[ignore = "manual product screenshot export for Plan 11 reflection-probe blending"]
fn export_runtime_shader_probe_blend_boundary_png() {
    let fixture = ProbeProductFixture::new("probe_blend_export");
    let frame = fixture.render_direct(fixture.environment_with_probes());

    let output = shader_test_output_dir()
        .join("runtime_shader_pbr_reflection_probe_blend_boundary_20260710.png");
    ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("probe blend frame dimensions")
        .save_with_format(output, ImageFormat::Png)
        .expect("write reflection-probe blend screenshot");
    assert_probe_boundary_is_smooth(frame.width, frame.height, &frame.rgba);

    fixture.cleanup();
}

struct ProbeProductFixture {
    root: PathBuf,
    asset_manager: Arc<ProjectAssetManager>,
    world: World,
    left_cubemap: ResourceId,
    right_cubemap: ResourceId,
}

impl ProbeProductFixture {
    fn new(label: &str) -> Self {
        let root = unique_temp_project_root(label);
        let paths = ProjectPaths::from_root(&root).expect("probe product paths");
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .expect("probe product layout");
        write_probe_product_project(&paths);

        let mut project = ProjectManager::open(&root).expect("probe product project manager");
        project
            .scan_and_import()
            .expect("import probe product model and material");
        write_probe_scene(probe_scene_path(&paths), &project);
        project
            .scan_and_import()
            .expect("import probe product scene");
        let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
        asset_manager
            .open_project(root.to_string_lossy().as_ref())
            .expect("open imported probe product project");
        let scene_uri = AssetUri::parse("res://scenes/reflection_probe.scene.toml")
            .expect("probe product scene URI");
        let world =
            World::load_scene_from_uri(&project, &scene_uri).expect("load probe product scene");
        let left_cubemap = register_probe_pmrem(
            &asset_manager,
            "res://generated/probe-left.zpmrem",
            LEFT_PROBE_PMREM_COLOR,
        );
        let right_cubemap = register_probe_pmrem(
            &asset_manager,
            "res://generated/probe-right.zpmrem",
            RIGHT_PROBE_PMREM_COLOR,
        );

        Self {
            root,
            asset_manager,
            world,
            left_cubemap,
            right_cubemap,
        }
    }

    fn environment_with_probes(&self) -> EnvironmentExtract {
        EnvironmentExtract::procedural_default().with_reflection_probes(vec![
            reflection_probe(1, -2.0, self.left_cubemap),
            reflection_probe(2, 2.0, self.right_cubemap),
        ])
    }

    fn snapshot(&self, with_probes: bool) -> crate::core::framework::render::RenderSceneSnapshot {
        let environment = if with_probes {
            self.environment_with_probes()
        } else {
            EnvironmentExtract::procedural_default()
        };
        let mut snapshot = self
            .world
            .build_viewport_render_packet(&SceneViewportExtractRequest {
                settings: ViewportRenderSettings::default(),
                active_camera_override: None,
                camera: None,
                viewport_size: Some(PROBE_PRODUCT_OUTPUT_SIZE),
                virtual_geometry_debug: None,
            });
        snapshot.environment = environment;
        snapshot.preview =
            PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
        snapshot.overlays = RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        };
        assert_eq!(
            snapshot.scene.camera.projection_mode,
            ProjectionMode::Orthographic
        );
        assert_eq!(
            snapshot.scene.camera.core_pipeline,
            CorePipelineKind::Core3d
        );
        snapshot
    }

    fn render_direct(&self, environment: EnvironmentExtract) -> ViewportFrame {
        let mut snapshot = self.snapshot(false);
        snapshot.environment = environment;
        snapshot.preview =
            PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
        let mut renderer = SceneRenderer::new_for_test_with_plugin_render_features(
            Arc::clone(&self.asset_manager),
            [reflection_probe_render_feature_descriptor()],
            Vec::new(),
        )
        .expect("probe product scene renderer");
        let frame = renderer
            .render(snapshot, PROBE_PRODUCT_OUTPUT_SIZE)
            .expect("render probe product frame");
        let diagnostics = renderer.reflection_probe_upload_diagnostics_for_tests();
        assert_eq!(
            diagnostics,
            (2, 2, 2, 0, None),
            "product probe frame must upload both PMREM assets before shading"
        );
        let (probe_count, positions, first_texels) = renderer
            .reflection_probe_gpu_upload_diagnostics_for_tests()
            .expect("read back product probe GPU resources");
        assert_eq!(probe_count, 2, "GPU probe header must expose both probes");
        assert_eq!(positions[0], [-2.0, 0.0, 0.0, 4.0]);
        assert_eq!(positions[1], [2.0, 0.0, 0.0, 4.0]);
        assert!(
            first_texels[0][0] > first_texels[0][2] && first_texels[1][2] > first_texels[1][0],
            "GPU probe array must preserve left-red/right-blue identity: {first_texels:?}"
        );
        frame
    }

    fn capture_with_framework(
        &self,
        framework: &WgpuRenderFramework,
        viewport: crate::core::framework::render::RenderViewportHandle,
        reflection_probes: bool,
        with_probes: bool,
        world_snapshot: u64,
    ) -> CapturedFrame {
        framework
            .set_quality_profile(viewport, probe_product_quality_profile(reflection_probes))
            .expect("set probe product quality profile");
        framework
            .submit_frame_extract(
                viewport,
                RenderFrameExtract::from_snapshot(
                    RenderWorldSnapshotHandle::new(world_snapshot),
                    self.snapshot(with_probes),
                ),
            )
            .expect("submit probe product frame");
        let stats = framework
            .query_stats()
            .expect("query probe product render stats");
        let compiled_probe_feature = stats
            .last_effective_features
            .iter()
            .any(|feature| feature == "reflection_probes");
        assert_eq!(
            compiled_probe_feature,
            reflection_probes,
            "compiled reflection-probe feature state must follow the quality profile: features={:?}, passes={:?}, executors={:?}, visible={}, mesh_draws={}, materials_ready={}",
            stats.last_effective_features,
            stats.last_graph_executed_passes,
            stats.last_graph_executed_executor_ids,
            stats.last_visibility_visible_count,
            stats.last_mesh_draw_count,
            stats.last_material_ready_count,
        );
        assert!(
            stats
                .last_graph_executed_passes
                .iter()
                .all(|pass| !pass.contains("reflection-probe")),
            "reflection probes shade through environment bindings; graph must not contain an unrequested capture/composite pass: {:?}",
            stats.last_graph_executed_passes,
        );
        assert!(
            stats.last_visibility_visible_count > 0
                && stats.last_mesh_draw_count > 0
                && stats.last_material_ready_count > 0,
            "probe product must draw a ready mirror surface: visible={}, mesh_draws={}, materials_ready={}, material_validation_errors={}, material_fallbacks={}, shader_misses={:?}",
            stats.last_visibility_visible_count,
            stats.last_mesh_draw_count,
            stats.last_material_ready_count,
            stats.last_material_validation_error_count,
            stats.last_material_fallback_count,
            stats.last_shader_variant_miss_report,
        );
        assert_framework_probe_upload(framework, reflection_probes, with_probes);
        framework
            .capture_frame(viewport)
            .expect("capture probe product frame")
            .expect("probe product frame should be available")
    }

    fn cleanup(self) {
        let _ = fs::remove_dir_all(self.root);
    }
}

fn assert_framework_probe_upload(
    framework: &WgpuRenderFramework,
    reflection_probes: bool,
    with_probes: bool,
) {
    let expected_extracted = if with_probes { 2 } else { 0 };
    let expected_active = if reflection_probes && with_probes {
        2
    } else {
        0
    };
    let diagnostics = framework.reflection_probe_upload_diagnostics_for_tests();
    assert_eq!(
        diagnostics.0, expected_extracted,
        "framework probe upload must receive the authored environment probes: {diagnostics:?}"
    );
    assert_eq!(
        diagnostics.1, expected_active,
        "framework probe upload activation must follow the compiled feature: {diagnostics:?}"
    );
    assert_eq!(diagnostics.3, 0, "probe PMREM assets must not be rejected");

    let (probe_count, positions, first_texels) = framework
        .reflection_probe_gpu_upload_diagnostics_for_tests()
        .expect("read back framework probe GPU resources");
    assert_eq!(
        probe_count, expected_active as u32,
        "GPU probe header must match the framework feature state"
    );
    if expected_active == 2 {
        assert_eq!(diagnostics.2, 2, "both probe PMREMs must be uploaded");
        assert_eq!(positions[0], [-2.0, 0.0, 0.0, 4.0]);
        assert_eq!(positions[1], [2.0, 0.0, 0.0, 4.0]);
        assert!(
            first_texels[0][0] > first_texels[0][2] && first_texels[1][2] > first_texels[1][0],
            "framework probe array must preserve left-red/right-blue identity: {first_texels:?}"
        );
    }
}

fn register_probe_pmrem(
    asset_manager: &ProjectAssetManager,
    uri_text: &str,
    color: [f32; 4],
) -> ResourceId {
    let uri = AssetUri::parse(uri_text).expect("probe PMREM URI");
    let id = ResourceId::from_locator(&uri);
    let source = build_source_cubemap_from_equirect(128, |_, _| color);
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
        .expect("probe PMREM payload");
    let texture = texture_asset_from_ibl_bake_artifact_pmrem(
        uri.clone(),
        &IblBakeArtifactBlob::from_payload(payload),
    )
    .expect("probe PMREM texture");
    asset_manager
        .assets::<TextureAsset>()
        .insert(ResourceRecord::new(id, ResourceKind::Texture, uri), texture)
        .expect("register probe PMREM texture");
    id
}

fn reflection_probe(probe_id: u64, center_x: f32, cubemap: ResourceId) -> ReflectionProbeData {
    ReflectionProbeData::try_new(
        probe_id,
        Vec3::new(center_x, 0.0, 0.0),
        Quat::IDENTITY,
        ProbeInfluenceShape::box_shape(Vec3::new(4.0, 10.0, 10.0), 4.0)
            .expect("probe product influence"),
        Vec3::new(4.0, 10.0, 10.0),
    )
    .expect("probe product contract")
    .with_baked_cubemap(Some(cubemap))
}

fn probe_product_quality_profile(reflection_probes: bool) -> RenderQualityProfile {
    RenderQualityProfile::new("plan11-reflection-probe-product")
        .with_reflection_probes(reflection_probes)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_clustered_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(false)
        .with_solari(false)
}

fn reflection_probe_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        Vec::new(),
    )
}

fn write_probe_product_project(paths: &ProjectPaths) {
    ProjectManifest::new(
        "ReflectionProbeProduct",
        AssetUri::parse("res://scenes/reflection_probe.scene.toml")
            .expect("probe product startup scene"),
        1,
    )
    .save(paths.manifest_path())
    .expect("write probe product manifest");
    write_probe_plane_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("probe_plane.obj"),
    );
    write_probe_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("probe_mirror.zmaterial"),
    );
}

fn probe_scene_path(paths: &ProjectPaths) -> PathBuf {
    paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("scenes")
        .join("reflection_probe.scene.toml")
}

fn write_probe_plane_obj(path: PathBuf) {
    fs::create_dir_all(path.parent().expect("probe plane parent"))
        .expect("create probe plane directory");
    let source = format!(
        "v {} {} 0\nv {} {} 0\nv {} {} 0\nv {} {} 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n",
        -PROBE_PLANE_HALF_WIDTH,
        -PROBE_PLANE_HALF_HEIGHT,
        PROBE_PLANE_HALF_WIDTH,
        -PROBE_PLANE_HALF_HEIGHT,
        PROBE_PLANE_HALF_WIDTH,
        PROBE_PLANE_HALF_HEIGHT,
        -PROBE_PLANE_HALF_WIDTH,
        PROBE_PLANE_HALF_HEIGHT,
    );
    fs::write(path, source).expect("write probe plane OBJ");
}

fn write_probe_material(path: PathBuf) {
    fs::create_dir_all(path.parent().expect("probe material parent"))
        .expect("create probe material directory");
    let mut material = MaterialAsset {
        name: Some("Reflection Probe Mirror".to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.92, 0.92, 0.92, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 1.0,
        roughness: 0.08,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    fs::write(
        path,
        material
            .to_project_toml_string(|reference| {
                if reference.locator.scheme()
                    == zircon_runtime_interface::resource::ResourceScheme::Builtin
                {
                    Ok(
                        zircon_runtime_interface::project::PersistedAssetReference::builtin(
                            reference.locator.clone(),
                        ),
                    )
                } else {
                    Err(crate::asset::ReferenceResolutionError::Registry {
                        message: "probe material project reference requires registry resolution"
                            .to_string(),
                    })
                }
            })
            .expect("probe material project TOML"),
    )
    .expect("write probe material");
}

fn write_probe_scene(path: PathBuf, project: &ProjectManager) {
    fs::create_dir_all(path.parent().expect("probe scene parent"))
        .expect("create probe scene directory");
    let probe_model = project_asset_reference(project, "res://models/probe_plane.obj");
    let probe_material = project_asset_reference(project, "res://materials/probe_mirror.zmaterial");
    let camera = SceneEntityAsset {
        entity: 1,
        name: "Camera".to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 0.0, 8.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 1,
        mobility: SceneMobilityAsset::Dynamic,
        camera: Some(SceneCameraAsset {
            projection_mode: ProjectionMode::Orthographic,
            ortho_size: PROBE_PRODUCT_ORTHO_SIZE,
            z_near: 0.1,
            z_far: 100.0,
            post_process_settings: None,
            ..SceneCameraAsset::default()
        }),
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    };
    let mirror = SceneEntityAsset {
        entity: 2,
        name: "Probe Blend Mirror".to_string(),
        parent: None,
        transform: TransformAsset::default(),
        active: true,
        render_layer_mask: 1,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: Some(SceneMeshInstanceAsset {
            model: probe_model,
            mesh: None,
            material: probe_material,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            depth_bias: 0.0,
            morph_weights: Vec::new(),
            primitives: Vec::new(),
            lods: Vec::new(),
        }),
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    };
    fs::write(
        path,
        SceneAsset {
            entities: vec![camera, mirror],
        }
        .to_project_toml_string(|reference| {
            project
                .persist_runtime_reference(reference)
                .map_err(|error| crate::asset::ReferenceResolutionError::Registry {
                    message: error.to_string(),
                })
        })
        .expect("probe scene project TOML"),
    )
    .expect("write probe product scene");
}

fn project_asset_reference(project: &ProjectManager, uri: &str) -> AssetReference {
    let locator = AssetUri::parse(uri).expect("probe project asset URI");
    let entry = project
        .asset_registry()
        .entry_by_path(&locator)
        .unwrap_or_else(|| panic!("probe project asset is not registered: {locator}"));
    AssetReference::new(entry.uuid(), locator)
}

fn assert_probe_boundary_is_smooth(width: u32, height: u32, rgba: &[u8]) {
    let mut blue_shares = Vec::new();
    for step in 0..17_u32 {
        let world_x = -2.0 + step as f32 * 0.25;
        let rgb = sample_probe_plane_rgb(width, height, rgba, world_x);
        blue_shares.push(rgb[2] / (rgb[0] + rgb[2]).max(1.0));
    }

    assert!(
        blue_shares[0] < 0.3 && blue_shares[16] > 0.7 && blue_shares[16] - blue_shares[0] > 0.4,
        "probe boundary endpoints must retain left/right reflection identity: {blue_shares:?}"
    );
    for pair in blue_shares.windows(2) {
        let delta = pair[1] - pair[0];
        assert!(
            delta >= -0.03,
            "probe blend must be monotonic across the boundary: {blue_shares:?}"
        );
        assert!(
            delta <= 0.16,
            "probe blend must not contain an abrupt boundary jump: {blue_shares:?}"
        );
    }
}

fn sample_probe_plane_rgb(width: u32, height: u32, rgba: &[u8], world_x: f32) -> [f32; 3] {
    let aspect = width as f32 / height.max(1) as f32;
    let world_width = PROBE_PRODUCT_ORTHO_SIZE * 2.0 * aspect;
    let center_x = ((world_x / world_width + 0.5) * width as f32).round() as i32;
    let center_y = (height / 2) as i32;
    let mut sum = [0.0; 3];
    let mut count = 0.0;
    for y in center_y - 3..=center_y + 3 {
        for x in center_x - 3..=center_x + 3 {
            let index = (y as usize * width as usize + x as usize) * 4;
            sum[0] += rgba[index] as f32;
            sum[1] += rgba[index + 1] as f32;
            sum[2] += rgba[index + 2] as f32;
            count += 1.0;
        }
    }
    [sum[0] / count, sum[1] / count, sum[2] / count]
}

fn mean_absolute_rgb_error(left: &[u8], right: &[u8]) -> f32 {
    assert_eq!(left.len(), right.len());
    let mut error = 0.0;
    let mut channels = 0_usize;
    for (left, right) in left.chunks_exact(4).zip(right.chunks_exact(4)) {
        for channel in 0..3 {
            error += (left[channel] as f32 - right[channel] as f32).abs();
            channels += 1;
        }
    }
    error / channels.max(1) as f32
}
