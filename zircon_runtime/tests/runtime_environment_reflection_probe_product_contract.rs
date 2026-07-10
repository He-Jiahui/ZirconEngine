use std::sync::Arc;

use zircon_runtime::asset::{
    texture_asset_from_ibl_bake_artifact_pmrem, AlphaMode, AssetReference, AssetUri, MaterialAsset,
    ProjectAssetManager, TextureAsset,
};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, CapturedFrame, DisplayMode, EnvironmentExtract,
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, ProbeInfluenceShape, ProceduralSkyParams, ProjectionMode,
    ReflectionProbeData, RenderFrameExtract, RenderFramework, RenderLayerSet, RenderMeshSnapshot,
    RenderOverlayExtract, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Quat, Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, WgpuRenderFramework,
};
use zircon_runtime::render_graph::QueueLane;

const OUTPUT_SIZE: UVec2 = UVec2::new(320, 240);
const LEFT_PMREM_COLOR: [f32; 4] = [0.9, 0.01, 0.01, 1.0];
const RIGHT_PMREM_COLOR: [f32; 4] = [0.01, 0.01, 0.9, 1.0];

#[test]
fn reflection_probe_feature_off_matches_skybox_and_enabled_probes_change_pixels() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_mirror_material(&asset_manager);
    let left = register_probe_pmrem(
        &asset_manager,
        "res://generated/integration-probe-left.zpmrem",
        LEFT_PMREM_COLOR,
    );
    let right = register_probe_pmrem(
        &asset_manager,
        "res://generated/integration-probe-right.zpmrem",
        RIGHT_PMREM_COLOR,
    );
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        Arc::clone(&asset_manager),
        [reflection_probe_render_feature_descriptor()],
        Vec::new(),
        Vec::new(),
    )
    .expect("reflection-probe product framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(OUTPUT_SIZE))
        .expect("reflection-probe product viewport");

    let probes = vec![
        reflection_probe(1, -2.0, left),
        reflection_probe(2, 2.0, right),
    ];
    let enabled = capture(
        &framework,
        viewport,
        material,
        true,
        EnvironmentExtract::procedural_default().with_reflection_probes(probes.clone()),
        1,
    );
    let disabled = capture(
        &framework,
        viewport,
        material,
        false,
        EnvironmentExtract::procedural_default().with_reflection_probes(probes),
        2,
    );
    let sky_only = capture(
        &framework,
        viewport,
        material,
        true,
        EnvironmentExtract::procedural_default(),
        3,
    );

    let fallback_error = mean_absolute_rgb_error(&disabled.rgba, &sky_only.rgba);
    let probe_difference = mean_absolute_rgb_error(&enabled.rgba, &sky_only.rgba);
    println!(
        "reflection_probe_product fallback_mae={fallback_error:.6} enabled_vs_sky_mae={probe_difference:.6}"
    );
    assert!(
        fallback_error <= 0.25,
        "feature-off probe frame must match sky fallback, MAE={fallback_error}"
    );
    assert!(
        probe_difference >= 4.0,
        "enabled probes must visibly differ from sky fallback, MAE={probe_difference}"
    );
}

fn capture(
    framework: &WgpuRenderFramework,
    viewport: zircon_runtime::core::framework::render::RenderViewportHandle,
    material: ResourceId,
    reflection_probes: bool,
    environment: EnvironmentExtract,
    world_snapshot: u64,
) -> CapturedFrame {
    framework
        .set_quality_profile(viewport, quality_profile(reflection_probes))
        .expect("set reflection-probe product quality");
    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(world_snapshot),
                scene_snapshot(material, environment),
            ),
        )
        .expect("submit reflection-probe product frame");

    let stats = framework
        .query_stats()
        .expect("reflection-probe product stats");
    let compiled_probe_feature = stats
        .last_effective_features
        .iter()
        .any(|feature| feature == "reflection_probes");
    assert_eq!(compiled_probe_feature, reflection_probes);
    assert!(stats.last_visibility_visible_count > 0);
    assert!(stats.last_mesh_draw_count > 0);
    assert!(stats.last_material_ready_count > 0);

    framework
        .capture_frame(viewport)
        .expect("capture reflection-probe product frame")
        .expect("reflection-probe product frame should be available")
}

fn scene_snapshot(material: ResourceId, environment: EnvironmentExtract) -> RenderSceneSnapshot {
    let camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 8.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 1.2,
        z_near: 0.1,
        z_far: 100.0,
        ..ViewportCameraSnapshot::default()
    };
    let preview =
        zircon_runtime::core::framework::render::PreviewEnvironmentExtract::from_environment(
            &environment,
            true,
            Vec4::ZERO,
        );

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: Transform::default(),
                model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                    "builtin://cube",
                )),
                mesh: None,
                material: ResourceHandle::<MaterialMarker>::new(material),
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
            }],
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
        environment,
        preview,
        virtual_geometry_debug: None,
    }
}

fn register_mirror_material(asset_manager: &ProjectAssetManager) -> ResourceId {
    let uri = AssetUri::parse("res://generated/integration-probe-mirror.zmaterial")
        .expect("mirror material URI");
    let id = ResourceId::from_locator(&uri);
    let mut material = MaterialAsset {
        name: Some("Reflection Probe Integration Mirror".to_string()),
        shader: AssetReference::from_locator(
            AssetUri::parse("builtin://shader/pbr.wgsl").expect("PBR shader URI"),
        ),
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
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri),
            material,
        )
        .expect("register mirror material");
    id
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
            .expect("reflection-probe product influence"),
        Vec3::new(4.0, 10.0, 10.0),
    )
    .expect("reflection-probe product contract")
    .with_baked_cubemap(Some(cubemap))
}

fn quality_profile(reflection_probes: bool) -> RenderQualityProfile {
    RenderQualityProfile::new("reflection-probe-public-product")
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
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn mean_absolute_rgb_error(left: &[u8], right: &[u8]) -> f32 {
    assert_eq!(left.len(), right.len());
    let mut total = 0_u64;
    let mut samples = 0_u64;
    for (left_pixel, right_pixel) in left.chunks_exact(4).zip(right.chunks_exact(4)) {
        for channel in 0..3 {
            total += (i16::from(left_pixel[channel]) - i16::from(right_pixel[channel]))
                .unsigned_abs() as u64;
            samples += 1;
        }
    }
    total as f32 / samples.max(1) as f32
}
