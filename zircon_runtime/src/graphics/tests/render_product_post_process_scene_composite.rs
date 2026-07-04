use std::{collections::BTreeMap, sync::Arc};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PostProcessGraphResourceNames, PreviewEnvironmentExtract,
    RenderDirectionalLightSnapshot, RenderFogSettings, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMeshSnapshot, RenderOverlayExtract, RenderPostProcessEffectStackSettings,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderScreenSpaceReflectionSettings, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

const POST_SCENE_COMPOSITE_EXECUTOR_ID: &str = "post.scene-composite";
const POST_SSR_REFLECTION_PYRAMID_EXECUTOR_ID: &str =
    "post.screen-space-reflection-reflection-pyramid";
const POST_SSR_REFLECTION_PYRAMID_COARSE_EXECUTOR_ID: &str =
    "post.screen-space-reflection-reflection-pyramid-coarse";
const POST_SSR_SPECULAR_OCCLUSION_EXECUTOR_ID: &str =
    "post.screen-space-reflection-specular-occlusion";
const POST_SSR_RESOLVE_EXECUTOR_ID: &str = "post.screen-space-reflection-resolve";

#[test]
fn render_product_post_scene_composite_fog_changes_final_frame() {
    let viewport_size = UVec2::new(128, 96);
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let baseline_viewport =
        create_scene_composite_product_viewport(&framework, viewport_size, "post-fog-baseline");
    let fog_viewport =
        create_scene_composite_product_viewport(&framework, viewport_size, "post-fog-scene");

    let (baseline, _) = submit_and_capture_scene_composite_product(
        &framework,
        baseline_viewport,
        scene_composite_product_extract(
            viewport_size,
            RenderPostProcessEffectStackSettings::default(),
        ),
    );
    let (fog_frame, stats) = submit_and_capture_scene_composite_product(
        &framework,
        fog_viewport,
        scene_composite_product_extract(viewport_size, fog_effect_stack()),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "scene-composite");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, POST_SCENE_COMPOSITE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_graph_executor_order(&stats, POST_SCENE_COMPOSITE_EXECUTOR_ID, "post.uber");
    assert_graph_executor_order(&stats, "post.uber", "post.output-transfer");
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["fog".to_string()]
    );
    let effect_stack_report = &stats.last_post_process_effect_stack_report;
    assert!(
        effect_stack_report.missing_resources.is_empty(),
        "scene-composite fog product scene should not miss resources; report={effect_stack_report:?}"
    );
    assert_texture_backings_are_distinct(
        &stats,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        PostProcessGraphResourceNames::TONEMAPPED,
    );

    let baseline_luma = average_luma_in_region(&baseline, UVec2::new(0, 0), viewport_size);
    let fog_luma = average_luma_in_region(&fog_frame, UVec2::new(0, 0), viewport_size);
    let frame_delta = frame_rgb_abs_delta(&fog_frame, &baseline);

    assert!(
        baseline_luma > 40.0,
        "scene-composite baseline clear should be visible; luma={baseline_luma:.2}"
    );
    assert!(
        (fog_luma - baseline_luma).abs() > 12.0,
        "scene-composite fog should shift final-frame luma; baseline={baseline_luma:.2}, fog={fog_luma:.2}"
    );
    assert!(
        frame_delta > 20_000,
        "scene-composite fog should produce a measurable final-frame delta; delta={frame_delta}, aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

#[test]
fn render_product_post_scene_composite_ssr_changes_final_frame() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_scene_composite_product_material(
        asset_manager.as_ref(),
        "res://materials/post_scene_composite_ssr_receiver.zmaterial",
        scene_composite_product_material(
            "SceneCompositeSsrReceiver",
            [0.03, 0.035, 0.045, 1.0],
            1.0,
            0.04,
            [0.0, 0.0, 0.0],
            false,
            true,
        ),
    );
    let caster_material = register_scene_composite_product_material(
        asset_manager.as_ref(),
        "res://materials/post_scene_composite_ssr_caster.zmaterial",
        scene_composite_product_material(
            "SceneCompositeSsrCaster",
            [1.0, 0.12, 0.03, 1.0],
            0.0,
            0.24,
            [3.6, 0.28, 0.08],
            false,
            false,
        ),
    );
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let baseline_viewport =
        create_scene_composite_ssr_product_viewport(&framework, viewport_size, "post-ssr-baseline");
    let ssr_viewport =
        create_scene_composite_ssr_product_viewport(&framework, viewport_size, "post-ssr-scene");

    let (baseline, _) = submit_and_capture_scene_composite_product(
        &framework,
        baseline_viewport,
        scene_composite_ssr_product_extract(
            viewport_size,
            receiver_material,
            caster_material,
            RenderPostProcessEffectStackSettings::default(),
        ),
    );
    framework
        .submit_frame_extract(
            ssr_viewport,
            scene_composite_ssr_product_extract(
                viewport_size,
                receiver_material,
                caster_material,
                ssr_effect_stack(),
            ),
        )
        .unwrap();
    let (ssr_frame, stats) = submit_and_capture_scene_composite_product(
        &framework,
        ssr_viewport,
        scene_composite_ssr_product_extract(
            viewport_size,
            receiver_material,
            caster_material,
            ssr_effect_stack(),
        ),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "scene-composite");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, POST_SSR_REFLECTION_PYRAMID_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_SSR_REFLECTION_PYRAMID_COARSE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_SSR_SPECULAR_OCCLUSION_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_SSR_RESOLVE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_SCENE_COMPOSITE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_graph_executor_order(
        &stats,
        POST_SSR_REFLECTION_PYRAMID_EXECUTOR_ID,
        POST_SSR_RESOLVE_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_SSR_SPECULAR_OCCLUSION_EXECUTOR_ID,
        POST_SSR_RESOLVE_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_SSR_RESOLVE_EXECUTOR_ID,
        POST_SCENE_COMPOSITE_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_SCENE_COMPOSITE_EXECUTOR_ID, "post.uber");
    assert_graph_executor_order(&stats, "post.uber", "post.output-transfer");
    assert_post_process_family_active(&stats, "screen-space-reflection");
    let effect_stack_report = &stats.last_post_process_effect_stack_report;
    assert!(
        effect_stack_report.missing_resources.is_empty(),
        "scene-composite SSR product scene should not miss resources; report={effect_stack_report:?}"
    );
    assert_texture_backing_exists(
        &stats,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
    );
    assert_texture_backing_exists(
        &stats,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
    );
    assert_texture_backing_exists(
        &stats,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
    );
    assert_texture_backing_exists(
        &stats,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
    );
    assert_texture_backings_are_distinct(
        &stats,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        PostProcessGraphResourceNames::TONEMAPPED,
    );

    let sample_origin = UVec2::new(24, 32);
    let sample_size = UVec2::new(112, 72);
    let baseline_luma = average_luma_in_region(&baseline, sample_origin, sample_size);
    let ssr_luma = average_luma_in_region(&ssr_frame, sample_origin, sample_size);
    let region_delta =
        frame_rgb_abs_delta_in_region(&ssr_frame, &baseline, sample_origin, sample_size);
    let frame_delta = frame_rgb_abs_delta(&ssr_frame, &baseline);

    assert!(
        baseline_luma > 3.0,
        "SSR baseline scene should contain visible geometry; luma={baseline_luma:.2}"
    );
    assert!(
        region_delta > 4_000 && frame_delta > 6_000,
        "scene-composite SSR should visibly alter the final frame; baseline_luma={baseline_luma:.2} ssr_luma={ssr_luma:.2} region_delta={region_delta} frame_delta={frame_delta} aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn create_scene_composite_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> crate::core::framework::render::RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, scene_composite_product_profile(profile_name))
        .unwrap();
    viewport
}

fn create_scene_composite_ssr_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> crate::core::framework::render::RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            scene_composite_product_profile(profile_name).with_temporal_history(true),
        )
        .unwrap();
    viewport
}

fn scene_composite_product_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_anti_alias(false)
}

fn scene_composite_product_extract(
    viewport_size: UVec2,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(924),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.68, 0.74, 0.82, 1.0),
            },
            virtual_geometry_debug: None,
        },
    );
    extract.apply_viewport_size(viewport_size);
    extract.post_process.effect_stack = effect_stack;
    extract
}

fn scene_composite_ssr_product_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(925),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.2, 2.2),
                        Vec3::new(0.0, 0.0, 0.15),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    scene_composite_product_mesh(
                        925_100,
                        Transform {
                            scale: Vec3::new(3.2, 2.2, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    scene_composite_product_mesh(
                        925_101,
                        Transform {
                            translation: Vec3::new(0.0, 0.0, 0.58),
                            scale: Vec3::new(0.38, 0.38, 0.72),
                            ..Transform::default()
                        },
                        caster_material,
                    ),
                ],
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 925_200,
                    light_id: 925_200,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        DEFAULT_RENDER_LAYER_MASK,
                    ),
                    direction: Vec3::new(0.45, 0.25, -1.0).normalize(),
                    color: Vec3::ONE,
                    intensity: 1.2,
                    shadow: None,
                }],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.01, 0.012, 0.018, 1.0),
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size);
    extract.post_process.effect_stack = effect_stack;
    extract
}

fn fog_effect_stack() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        fog: RenderFogSettings {
            density: 0.85,
            height_falloff: 0.35,
            color: Vec3::new(0.02, 0.10, 0.82),
        },
        ..Default::default()
    }
}

fn ssr_effect_stack() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 8.0,
            thickness: 0.65,
            max_ray_distance: 80.0,
            max_steps: 96,
            temporal_blend_factor: 0.0,
            roughness_mip_bias: -1.0,
        },
        ..Default::default()
    }
}

fn scene_composite_product_mesh(
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
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
    }
}

fn scene_composite_product_material(
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    emissive: [f32; 3],
    cast_shadows: bool,
    receive_shadows: bool,
) -> MaterialAsset {
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "cast_shadows".to_string(),
        toml::Value::Boolean(cast_shadows),
    );
    property_values.insert(
        "receive_shadows".to_string(),
        toml::Value::Boolean(receive_shadows),
    );

    MaterialAsset {
        name: Some(name.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive,
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn register_scene_composite_product_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    material: MaterialAsset,
) -> ResourceId {
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("scene-composite product material insert");
    material_id
}

fn submit_and_capture_scene_composite_product(
    framework: &WgpuRenderFramework,
    viewport: crate::core::framework::render::RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("scene-composite product frame should be capturable");
    let stats = framework.query_stats().unwrap();
    (frame, stats)
}

fn assert_post_process_node_executed(stats: &RenderStats, node: &str) {
    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|executed| executed == node),
        "expected post-process node `{node}` to execute; executed={:?}; graph_executors={:?}; skipped_node_count={}; effect_stack_report={:?}",
        stats.last_post_process_graph_executed_nodes,
        stats.last_graph_executed_executor_ids,
        stats.last_post_process_graph_skipped_node_count,
        stats.last_post_process_effect_stack_report
    );
}

fn assert_graph_executor_executed(stats: &RenderStats, executor_id: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "expected graph executor `{executor_id}` to execute; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_graph_executor_order(stats: &RenderStats, before: &str, after: &str) {
    let before_index = graph_executor_index(stats, before);
    let after_index = graph_executor_index(stats, after);
    assert!(
        before_index < after_index,
        "expected graph executor `{before}` before `{after}`; executed={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_post_process_family_active(stats: &RenderStats, family: &str) {
    assert!(
        stats
            .last_post_process_effect_stack_report
            .active_families
            .iter()
            .any(|active_family| active_family == family),
        "expected post-process family `{family}` to be active; families={:?}",
        stats.last_post_process_effect_stack_report.active_families
    );
}

fn graph_executor_index(stats: &RenderStats, executor_id: &str) -> usize {
    stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executed| executed == executor_id)
        .unwrap_or_else(|| {
            panic!(
                "graph executor `{executor_id}` was not executed; executed={:?}",
                stats.last_graph_executed_executor_ids
            )
        })
}

fn assert_texture_backings_are_distinct(stats: &RenderStats, first: &str, second: &str) {
    let first_backing = texture_backing_for(stats, first);
    let second_backing = texture_backing_for(stats, second);
    assert_ne!(
        first_backing, second_backing,
        "expected `{first}` and `{second}` to use distinct texture backings; aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn assert_texture_backing_exists(stats: &RenderStats, resource_name: &str) {
    let _ = texture_backing_for(stats, resource_name);
}

fn texture_backing_for<'a>(stats: &'a RenderStats, resource_name: &str) -> &'a str {
    stats
        .last_graph_execution_alias_report
        .texture_aliases
        .iter()
        .find(|alias| alias.logical_name == resource_name)
        .map(|alias| alias.backing_name.as_str())
        .unwrap_or_else(|| {
            panic!(
                "missing texture alias for `{resource_name}`; aliases={:?}",
                stats.last_graph_execution_alias_report.texture_aliases
            )
        })
}

fn average_luma_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0.0f32;
    let max_y = (origin.y + size.y).min(frame.height);
    let max_x = (origin.x + size.x).min(frame.width);
    for y in origin.y..max_y {
        for x in origin.x..max_x {
            let index = ((y * frame.width + x) * 4) as usize;
            let r = frame.rgba[index] as f32;
            let g = frame.rgba[index + 1] as f32;
            let b = frame.rgba[index + 2] as f32;
            sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
            count += 1.0;
        }
    }
    sum / count.max(1.0)
}

fn frame_rgb_abs_delta(left: &CapturedFrame, right: &CapturedFrame) -> u64 {
    assert_eq!(left.width, right.width);
    assert_eq!(left.height, right.height);
    left.rgba
        .chunks_exact(4)
        .zip(right.rgba.chunks_exact(4))
        .map(|(left_px, right_px)| {
            left_px[0].abs_diff(right_px[0]) as u64
                + left_px[1].abs_diff(right_px[1]) as u64
                + left_px[2].abs_diff(right_px[2]) as u64
        })
        .sum()
}

fn frame_rgb_abs_delta_in_region(
    left: &CapturedFrame,
    right: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
) -> u64 {
    assert_eq!(left.width, right.width);
    assert_eq!(left.height, right.height);
    let max_y = (origin.y + size.y).min(left.height);
    let max_x = (origin.x + size.x).min(left.width);
    let mut delta = 0;
    for y in origin.y..max_y {
        for x in origin.x..max_x {
            let index = ((y * left.width + x) * 4) as usize;
            delta += left.rgba[index].abs_diff(right.rgba[index]) as u64;
            delta += left.rgba[index + 1].abs_diff(right.rgba[index + 1]) as u64;
            delta += left.rgba[index + 2].abs_diff(right.rgba[index + 2]) as u64;
        }
    }
    delta
}
