use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::AssetUri;
use crate::core::framework::render::{
    CorePipelineKind, DisplayMode, PostProcessGraphResourceNames, RenderFramework, RenderPhase,
    RenderPipelineHandle, RenderQualityProfile, RenderStats, RenderViewportDescriptor,
    ShaderFeatureBits, ShaderQualityTier, ShaderVariantMissReport,
    ShaderVariantPrewarmDimensionCount, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantRuntimeDimensionCount, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry, prewarm_shader_variants,
};
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset,
    RendererAsset, RendererFeatureAsset, WgpuRenderFramework,
};
use crate::render_graph::QueueLane;

use super::{
    register_material_revision, register_static_skinned_mesh_revision,
    register_static_skinned_skeleton_revision, static_cache_skinned_extract,
};

#[test]
fn render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss() {
    let cache_roots =
        shader_cache_test_roots("zircon_product_base_mesh_staged_prewarm_second_launch");
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let manifest = base_mesh_shader_cache_product_manifest();
    let prewarm_report = prewarm_shader_variants(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert_staged_prewarm_dimension_written(&prewarm_report);

    let first_launch = submit_base_mesh_with_staged_cache(
        1801,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
    );
    let second_launch = submit_base_mesh_with_staged_cache(
        1802,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
    );

    assert_staged_base_mesh_shader_cache_hit(
        &first_launch,
        "first product launch",
        &prewarm_report,
    );
    assert_staged_base_mesh_shader_cache_hit(
        &second_launch,
        "second product launch",
        &prewarm_report,
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
}

fn submit_base_mesh_with_staged_cache(
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
) -> RenderStats {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/staged-prewarm-product.zmaterial")
        .expect("staged prewarm product material uri");
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "staged-prewarm-product-v1",
    );
    let mesh_uri = AssetUri::parse("res://meshes/staged-prewarm-product-skinned.zmesh")
        .expect("staged prewarm product skinned mesh uri");
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    register_static_skinned_mesh_revision(
        &asset_manager,
        mesh_id,
        mesh_uri,
        "staged-prewarm-product-skinned-mesh-v1",
    );
    let skeleton_uri = AssetUri::parse("res://animation/staged-prewarm-product.skeleton.zranim")
        .expect("staged prewarm product skeleton uri");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    register_static_skinned_skeleton_revision(
        &asset_manager,
        skeleton_id,
        skeleton_uri,
        "staged-prewarm-product-skeleton-v1",
    );

    let framework = WgpuRenderFramework::new(asset_manager).expect("WGPU framework");
    framework.replace_shader_variant_disk_cache_for_tests(
        ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root]),
    );
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .expect("viewport");
    let pipeline = framework
        .register_pipeline_asset(base_mesh_shader_cache_product_pipeline())
        .expect("base mesh product pipeline");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("set base mesh product pipeline");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("staged-prewarm-product-base")
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_anti_alias(false),
        )
        .expect("quality profile");

    let mut extract = static_cache_skinned_extract(material_id, mesh_id, skeleton_id, world);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    framework
        .submit_frame_extract(viewport, extract)
        .expect("submit staged prewarm product extract");
    framework.query_stats().expect("render stats")
}

fn assert_staged_base_mesh_shader_cache_hit(
    stats: &RenderStats,
    launch_label: &str,
    prewarm_report: &ShaderVariantPrewarmReport,
) {
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        report.request_count >= 1,
        "{launch_label} should request at least one Base mesh shader variant; stats={stats:?}"
    );
    assert!(
        stats.last_mesh_replay_state_change_count >= 1,
        "{launch_label} should replay at least one mesh pipeline state change; stats={stats:?}"
    );
    assert!(
        stats.last_mesh_skinned_draw_count >= 1,
        "{launch_label} should exercise the product Base mesh path with a skinned direct replay fixture; stats={stats:?}"
    );
    assert!(
        report.disk_hit_count >= 1,
        "{launch_label} should consume the staged shader variant cache; stats={stats:?}"
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
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executor| executor == "mesh.opaque"),
        "{launch_label} should execute the product mesh Base/Opaque path"
    );
    assert_staged_prewarm_runtime_dimension_correlation(prewarm_report, report, launch_label);
}

fn assert_staged_prewarm_dimension_written(report: &ShaderVariantPrewarmReport) {
    let skinned_geometry = GEOMETRY_SOURCE_ID_SKINNED_MESH.value().to_string();
    let standard_pbr = SHADING_MODEL_ID_STANDARD_PBR.value().to_string();

    assert_prewarm_dimension_written(
        report.dimension_summary.pass_types.get("forward"),
        "prewarm forward pass",
    );
    assert_prewarm_dimension_written(
        report
            .dimension_summary
            .geometry_source_ids
            .get(&skinned_geometry),
        "prewarm skinned geometry source",
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

fn assert_staged_prewarm_runtime_dimension_correlation(
    prewarm_report: &ShaderVariantPrewarmReport,
    runtime_report: &ShaderVariantMissReport,
    launch_label: &str,
) {
    assert_staged_prewarm_dimension_written(prewarm_report);
    let skinned_geometry = GEOMETRY_SOURCE_ID_SKINNED_MESH.value().to_string();
    let standard_pbr = SHADING_MODEL_ID_STANDARD_PBR.value().to_string();

    assert_runtime_dimension_disk_hit(
        runtime_report.dimension_summary.pass_types.get("forward"),
        launch_label,
        "forward pass",
    );
    assert_runtime_dimension_disk_hit(
        runtime_report
            .dimension_summary
            .geometry_source_ids
            .get(&skinned_geometry),
        launch_label,
        "skinned geometry source",
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

fn base_mesh_shader_cache_product_manifest() -> ShaderVariantPrewarmManifest {
    let mut variants = builtin_fallback_shader_prewarm_manifest().variants;
    variants.extend(
        builtin_standard_material_shader_prewarm_manifest_for_geometry(
            ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
            SHADING_MODEL_ID_STANDARD_PBR,
            None,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            &[ShaderQualityTier::Medium],
        )
        .variants,
    );
    ShaderVariantPrewarmManifest::new(variants)
}

fn base_mesh_shader_cache_product_pipeline() -> RenderPipelineAsset {
    RenderPipelineAsset {
        handle: RenderPipelineHandle::new(808),
        revision: 1,
        name: "plan08-staged-prewarm-base-mesh-product".to_string(),
        core_pipeline: CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::Prepass, RenderPhase::Opaque3d],
        renderer: RendererAsset {
            name: "plan08-staged-prewarm-base-mesh-renderer".to_string(),
            stages: vec![RenderPassStage::DepthPrepass, RenderPassStage::Opaque3d],
            features: vec![RendererFeatureAsset::plugin(
                base_mesh_shader_cache_product_feature(),
            )],
        },
    }
}

fn base_mesh_shader_cache_product_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.staged_prewarm_base_mesh_product",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plan08-staged-prewarm-preview-clear",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-scene-color")
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque3d,
                "plan08-staged-prewarm-opaque-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.opaque")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

struct ShaderCacheTestRoots {
    root: PathBuf,
    runtime_root: PathBuf,
    staged_root: PathBuf,
}

fn shader_cache_test_roots(label: &str) -> ShaderCacheTestRoots {
    let root = std::env::temp_dir().join(format!("{label}_{}", std::process::id()));
    ShaderCacheTestRoots {
        runtime_root: root.join("runtime").join("shader_variants"),
        staged_root: root.join("staged").join("cache").join("shader_variants"),
        root,
    }
}
