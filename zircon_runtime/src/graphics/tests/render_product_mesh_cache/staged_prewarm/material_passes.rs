use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::asset::AssetUri;
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasSettings, CorePipelineKind, DEFAULT_RENDER_LAYER_MASK, DisplayMode,
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometryExtract, LightShadowSettings,
    PostProcessGraphResourceNames, ProjectionMode, RenderDirectionalLightSnapshot, RenderFramework,
    RenderLayerSet, RenderMeshSnapshot, RenderPhase, RenderPipelineHandle, RenderQualityProfile,
    RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    SHADING_MODEL_ID_STANDARD_PBR, ShaderFeatureBits, ShaderQualityTier, ShaderVariantMissReport,
    ShaderVariantPrewarmDimensionCount, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantRuntimeDimensionCount, ShadowPcfQuality, ShadowResolutionTier,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry, prewarm_shader_variants,
};
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
    RenderPipelineAsset, RendererAsset, RendererFeatureAsset, WgpuRenderFramework,
};
use crate::render_graph::QueueLane;

use super::super::super::render_product_submit::snapshot_with_projection_for_mesh_cache_tests;
use super::super::register_taa_reactive_material_revision;
use super::shader_cache_test_roots;

#[test]
fn render_product_material_mesh_passes_second_launch_use_staged_prewarm_without_compile_miss() {
    let cache_roots =
        shader_cache_test_roots("zircon_product_material_mesh_passes_staged_prewarm_second_launch");
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let manifest = material_mesh_shader_cache_product_manifest();
    let prewarm_report = prewarm_shader_variants(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert_material_mesh_staged_prewarm_dimensions_written(&prewarm_report);

    let first_launch = submit_material_mesh_passes_with_staged_cache(
        2801,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
    );
    let second_launch = submit_material_mesh_passes_with_staged_cache(
        2802,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
    );

    assert_staged_material_mesh_first_frame_shader_cache_hit(
        &first_launch.first_frame,
        "first product launch first frame",
        &prewarm_report,
    );
    assert_staged_material_mesh_velocity_frame_shader_cache_hit(
        &first_launch.velocity_frame,
        "first product launch velocity frame",
        &prewarm_report,
    );
    assert_staged_material_mesh_first_frame_shader_cache_hit(
        &second_launch.first_frame,
        "second product launch first frame",
        &prewarm_report,
    );
    assert_staged_material_mesh_velocity_frame_shader_cache_hit(
        &second_launch.velocity_frame,
        "second product launch velocity frame",
        &prewarm_report,
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
}

fn submit_material_mesh_passes_with_staged_cache(
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
) -> MaterialMeshPassLaunchStats {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/staged-prewarm-material-mesh.zmaterial")
        .expect("staged prewarm material mesh uri");
    let material_id = ResourceId::from_locator(&material_uri);
    register_taa_reactive_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "staged-prewarm-material-mesh-v1",
        1.0,
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).expect("WGPU framework");
    framework.replace_shader_variant_disk_cache_for_tests(
        ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root]),
    );
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .expect("viewport");
    let pipeline = framework
        .register_pipeline_asset(material_mesh_shader_cache_product_pipeline())
        .expect("material mesh product pipeline");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("set material mesh product pipeline");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("staged-prewarm-product-material-mesh")
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(true)
                .with_temporal_history(true)
                .with_anti_alias(true),
        )
        .expect("quality profile");

    let first_extract = material_mesh_cache_extract(material_id, world, 0.0);
    framework
        .submit_frame_extract(viewport, first_extract)
        .expect("submit material mesh first frame");
    let first_frame = framework.query_stats().expect("first frame stats");

    let velocity_extract = material_mesh_cache_extract(material_id, world + 10_000, 0.125);
    framework
        .submit_frame_extract(viewport, velocity_extract)
        .expect("submit material mesh velocity frame");
    let velocity_frame = framework.query_stats().expect("velocity frame stats");

    MaterialMeshPassLaunchStats {
        first_frame,
        velocity_frame,
    }
}

fn material_mesh_cache_extract(
    material_id: ResourceId,
    world: u64,
    x_offset: f32,
) -> crate::core::framework::render::RenderFrameExtract {
    let mut extract = crate::core::framework::render::RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![material_mesh_cache_mesh(material_id, x_offset)],
    );
    extract.lighting.directional_lights = vec![material_mesh_cache_shadow_light()];
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &extract.view.anti_alias);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    extract
}

fn material_mesh_cache_mesh(material_id: ResourceId, x_offset: f32) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 2_803,
        stable_instance_key: 2_803 << 16,
        transform_revision: 1,
        transform: Transform {
            translation: Vec3::new(x_offset, 0.0, 0.0),
            scale: Vec3::splat(0.8),
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            ..Default::default()
        },
    }
}

fn material_mesh_cache_shadow_light() -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 2_900,
        light_id: 2_900,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        direction: Vec3::new(0.35, -0.25, -1.0).normalize(),
        color: Vec3::ONE,
        intensity: 1.0,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: Some(LightShadowSettings {
            casts_shadow: true,
            depth_bias: 0.0,
            normal_bias: 0.0,
            strength: 1.0,
            resolution_preference: ShadowResolutionTier::T512,
            pcf_quality: ShadowPcfQuality::Medium,
        }),
    }
}

fn assert_staged_material_mesh_first_frame_shader_cache_hit(
    stats: &RenderStats,
    launch_label: &str,
    prewarm_report: &ShaderVariantPrewarmReport,
) {
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        stats.last_mesh_opaque_draw_count >= 1,
        "{launch_label} should exercise a product opaque material mesh; stats={stats:?}"
    );
    assert!(
        stats.last_mesh_shadow_caster_draw_count >= 1,
        "{launch_label} should exercise a product shadow-casting material mesh; stats={stats:?}"
    );
    assert_eq!(
        stats.last_mesh_taa_reactive_mask_command_count, 1,
        "{launch_label} should build the product TAA reactive material-mask command even when first-frame history falls back to terminal AA; stats={stats:?}"
    );
    assert_executed_product_material_mesh_executors(
        stats,
        launch_label,
        &[
            "deferred.depth-prepass",
            "deferred.gbuffer",
            "shadow.atlas",
            "lighting.light-grid",
            "lighting.deferred",
            "post.output-transfer",
        ],
    );
    assert!(
        report.request_count >= 4,
        "{launch_label} should request DepthPrepass/GBuffer/Shadow/TAA material mesh variants; stats={stats:?}"
    );
    assert_eq!(
        report.compile_miss_count, 0,
        "{launch_label} must not compile shader variants at product submit time; stats={stats:?}"
    );
    assert_eq!(
        report.disk_write_count, 0,
        "{launch_label} should not write runtime shader cache entries when staged cache hits; stats={stats:?}"
    );
    assert_eq!(
        report.disk_error_count, 0,
        "{launch_label} should not report shader cache disk errors; stats={stats:?}"
    );
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.pass_types.get("depth_prepass"),
        launch_label,
        "depth-prepass pass",
    );
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.pass_types.get("gbuffer"),
        launch_label,
        "gbuffer pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("shadow"),
        launch_label,
        "shadow pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("taa_reactive_mask"),
        launch_label,
        "TAA reactive mask pass",
    );
    assert_material_mesh_runtime_dimensions_hit(prewarm_report, report, launch_label);
}

fn assert_staged_material_mesh_velocity_frame_shader_cache_hit(
    stats: &RenderStats,
    launch_label: &str,
    prewarm_report: &ShaderVariantPrewarmReport,
) {
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        stats.last_mesh_opaque_draw_count >= 1,
        "{launch_label} should continue exercising the product opaque material mesh; stats={stats:?}"
    );
    assert_eq!(
        stats.last_mesh_taa_reactive_mask_command_count, 1,
        "{launch_label} should build the product TAA reactive material-mask command for the reactive material; stats={stats:?}"
    );
    assert!(
        stats.last_mesh_previous_velocity_transform_draw_count >= 1,
        "{launch_label} should use a previous velocity transform for the product material mesh; stats={stats:?}"
    );
    assert_eq!(
        stats.last_mesh_missing_velocity_transform_draw_count, 0,
        "{launch_label} should not miss previous velocity transforms for the product material mesh; stats={stats:?}"
    );
    assert_executed_product_material_mesh_executors(
        stats,
        launch_label,
        &[
            "deferred.depth-prepass",
            "deferred.gbuffer",
            "shadow.atlas",
            "lighting.light-grid",
            "lighting.deferred",
            "post.output-transfer",
        ],
    );
    assert_executed_product_material_mesh_executor(stats, launch_label, "temporal.velocity-object");
    assert!(
        report.request_count >= 5,
        "{launch_label} should request repeat material mesh variants including the velocity pass without compile misses; stats={stats:?}"
    );
    assert_eq!(
        report.compile_miss_count, 0,
        "{launch_label} must not compile shader variants at product submit time; stats={stats:?}"
    );
    assert_eq!(
        report.disk_write_count, 0,
        "{launch_label} should not write runtime shader cache entries when staged cache hits; stats={stats:?}"
    );
    assert_eq!(
        report.disk_error_count, 0,
        "{launch_label} should not report shader cache disk errors; stats={stats:?}"
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("shadow"),
        launch_label,
        "shadow pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("taa_reactive_mask"),
        launch_label,
        "TAA reactive mask pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("velocity"),
        launch_label,
        "velocity pass",
    );
    assert_material_mesh_runtime_dimensions_requested_without_miss(
        prewarm_report,
        report,
        launch_label,
    );
}

fn assert_executed_product_material_mesh_executors(
    stats: &RenderStats,
    launch_label: &str,
    executor_ids: &[&str],
) {
    for executor_id in executor_ids {
        assert_executed_product_material_mesh_executor(stats, launch_label, executor_id);
    }
}

fn assert_executed_product_material_mesh_executor(
    stats: &RenderStats,
    launch_label: &str,
    executor_id: &str,
) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executor| executor == executor_id),
        "{launch_label} should execute product executor `{executor_id}`; pipeline={:?}; features={:?}; pass_count={}; culled_pass_count={}; executed={:?}",
        stats.last_pipeline,
        stats.last_effective_features,
        stats.last_graph_pass_count,
        stats.last_graph_culled_pass_count,
        stats.last_graph_executed_executor_ids
    );
}

fn assert_material_mesh_staged_prewarm_dimensions_written(report: &ShaderVariantPrewarmReport) {
    let static_geometry = GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string();
    let standard_pbr = SHADING_MODEL_ID_STANDARD_PBR.value().to_string();

    for (pass_type, label) in [
        ("forward", "prewarm forward pass"),
        ("gbuffer", "prewarm gbuffer pass"),
        ("depth_prepass", "prewarm depth-prepass pass"),
        ("shadow", "prewarm shadow pass"),
        ("velocity", "prewarm velocity pass"),
        ("taa_reactive_mask", "prewarm TAA reactive mask pass"),
    ] {
        assert_prewarm_dimension_written(report.dimension_summary.pass_types.get(pass_type), label);
    }
    assert_prewarm_dimension_written(
        report
            .dimension_summary
            .geometry_source_ids
            .get(&static_geometry),
        "prewarm static geometry source",
    );
    assert_prewarm_dimension_written(
        report
            .dimension_summary
            .shading_model_ids
            .get(&standard_pbr),
        "prewarm StandardPBR shading model",
    );
    assert_prewarm_dimension_written(
        report.dimension_summary.quality_tiers.get("medium"),
        "prewarm medium quality tier",
    );
}

fn assert_material_mesh_runtime_dimensions_hit(
    prewarm_report: &ShaderVariantPrewarmReport,
    runtime_report: &ShaderVariantMissReport,
    launch_label: &str,
) {
    assert_material_mesh_staged_prewarm_dimensions_written(prewarm_report);
    let static_geometry = GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string();
    let standard_pbr = SHADING_MODEL_ID_STANDARD_PBR.value().to_string();

    assert_runtime_dimension_disk_hit(
        runtime_report
            .dimension_summary
            .geometry_source_ids
            .get(&static_geometry),
        launch_label,
        "static geometry source",
    );
    assert_runtime_dimension_disk_hit(
        runtime_report
            .dimension_summary
            .shading_model_ids
            .get(&standard_pbr),
        launch_label,
        "StandardPBR shading model",
    );
    assert_runtime_dimension_disk_hit(
        runtime_report.dimension_summary.quality_tiers.get("medium"),
        launch_label,
        "medium quality tier",
    );
}

fn assert_material_mesh_runtime_dimensions_requested_without_miss(
    prewarm_report: &ShaderVariantPrewarmReport,
    runtime_report: &ShaderVariantMissReport,
    launch_label: &str,
) {
    assert_material_mesh_staged_prewarm_dimensions_written(prewarm_report);
    let static_geometry = GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string();
    let standard_pbr = SHADING_MODEL_ID_STANDARD_PBR.value().to_string();

    assert_runtime_dimension_requested_without_miss(
        runtime_report
            .dimension_summary
            .geometry_source_ids
            .get(&static_geometry),
        launch_label,
        "static geometry source",
    );
    assert_runtime_dimension_requested_without_miss(
        runtime_report
            .dimension_summary
            .shading_model_ids
            .get(&standard_pbr),
        launch_label,
        "StandardPBR shading model",
    );
    assert_runtime_dimension_requested_without_miss(
        runtime_report.dimension_summary.quality_tiers.get("medium"),
        launch_label,
        "medium quality tier",
    );
}

fn assert_prewarm_dimension_written(
    count: Option<&ShaderVariantPrewarmDimensionCount>,
    label: &str,
) {
    let count = count.unwrap_or_else(|| panic!("{label} should be present in prewarm report"));
    assert!(
        count.written_count >= 1,
        "{label} should include at least one written prewarm variant; count={count:?}"
    );
}

fn assert_runtime_dimension_disk_hit(
    count: Option<&ShaderVariantRuntimeDimensionCount>,
    launch_label: &str,
    dimension_label: &str,
) {
    let count = count.unwrap_or_else(|| {
        panic!("{launch_label} should report runtime dimension {dimension_label}")
    });
    assert!(
        count.disk_hit_count >= 1,
        "{launch_label} should disk-hit staged cache for {dimension_label}; count={count:?}"
    );
    assert_eq!(
        count.compile_miss_count, 0,
        "{launch_label} should not compile-miss staged cache for {dimension_label}; count={count:?}"
    );
}

fn assert_runtime_dimension_requested_without_miss(
    count: Option<&ShaderVariantRuntimeDimensionCount>,
    launch_label: &str,
    dimension_label: &str,
) {
    let count = count.unwrap_or_else(|| {
        panic!("{launch_label} should report runtime dimension {dimension_label}")
    });
    assert!(
        count.request_count >= 1,
        "{launch_label} should request {dimension_label}; count={count:?}"
    );
    assert_eq!(
        count.compile_miss_count, 0,
        "{launch_label} should not compile-miss staged cache for {dimension_label}; count={count:?}"
    );
}

fn material_mesh_shader_cache_product_manifest() -> ShaderVariantPrewarmManifest {
    let mut variants = builtin_fallback_shader_prewarm_manifest().variants;
    variants.extend(
        builtin_standard_material_shader_prewarm_manifest_for_geometry(
            ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
            SHADING_MODEL_ID_STANDARD_PBR,
            None,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            &[ShaderQualityTier::Medium],
        )
        .variants,
    );
    ShaderVariantPrewarmManifest::new(variants)
}

fn material_mesh_shader_cache_product_pipeline() -> RenderPipelineAsset {
    RenderPipelineAsset {
        handle: RenderPipelineHandle::new(809),
        revision: 1,
        name: "plan08-staged-prewarm-material-mesh-product".to_string(),
        core_pipeline: CorePipelineKind::Core3d,
        phase_mapping: vec![
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Deferred,
            RenderPhase::PostProcess,
        ],
        renderer: RendererAsset {
            name: "plan08-staged-prewarm-material-mesh-renderer".to_string(),
            stages: vec![
                RenderPassStage::DepthPrepass,
                RenderPassStage::Shadow,
                RenderPassStage::Deferred,
                RenderPassStage::Lighting,
                RenderPassStage::PostProcess,
            ],
            features: vec![
                RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredGeometry)
                    .with_descriptor_override(material_mesh_shader_cache_product_feature())
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::Temporal)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredLighting)
                    .with_descriptor_override(material_mesh_deferred_lighting_product_feature())
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::AntiAlias)
                    .without_quality_gate(),
            ],
        },
    }
}

fn material_mesh_shader_cache_product_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.staged_prewarm_material_mesh_product",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plan08-staged-prewarm-depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Shadow,
                "plan08-staged-prewarm-shadow-atlas",
                QueueLane::Graphics,
            )
            .with_executor_id("shadow.atlas")
            .with_side_effects()
            .write_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Deferred,
                "plan08-staged-prewarm-gbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.gbuffer")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE),
        ],
    )
}

fn material_mesh_deferred_lighting_product_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.staged_prewarm_material_mesh_deferred_lighting",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
            "lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "plan08-staged-prewarm-deferred-lighting",
                QueueLane::Graphics,
            )
            .with_executor_id("lighting.deferred")
            .read_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .read_external_texture(PostProcessGraphResourceNames::FINAL_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

struct MaterialMeshPassLaunchStats {
    first_frame: RenderStats,
    velocity_frame: RenderStats,
}
