use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, ShaderAsset, ShaderSourceLanguage};
use crate::core::framework::render::{
    CorePipelineKind, DisplayMode, PostProcessGraphResourceNames, RenderFramework, RenderPhase,
    RenderPipelineHandle, RenderQualityProfile, RenderStats, RenderViewportDescriptor,
    ShaderAssetKind, ShaderFeatureBits, ShaderPassType, ShaderQualityTier,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport, ShaderVariantPrewarmRequest,
    GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};
use crate::dynamic_api::{
    builtin_standard_material_shader_prewarm_manifest_for_geometry, prewarm_shader_variants,
};
use crate::graphics::shader::ShaderVariantCacheDisk;
use crate::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset,
    RendererAsset, RendererFeatureAsset, WgpuRenderFramework,
};
use crate::render_graph::QueueLane;

use super::super::render_product_submit::material_with_import_note;
use super::{
    register_material_asset_revision, registry_staged_cache_runtime_surface_source,
    static_cache_extract,
};

const MATERIAL_SHADER_TEMPLATE_REVISION: &str = "zr-material-template-v1";

#[test]
fn render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss() {
    let cache_roots =
        shader_cache_test_roots("zircon_product_project_plugin_registry_staged_prewarm");
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let registry_cases = registry_shader_cases();
    let manifest = registry_product_prewarm_manifest(&registry_cases);
    let registry_shader_source = registry_staged_cache_runtime_surface_source();
    let prewarm_report = prewarm_shader_variants(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, registry_cases.len());
    assert_eq!(prewarm_report.written_count, registry_cases.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert_eq!(
        prewarm_report.source_provenance.source_count,
        registry_cases.len()
    );
    for case in registry_cases.iter().copied() {
        assert_registry_product_prewarm_written(&manifest, &prewarm_report, case);
    }

    for (index, case) in registry_cases.iter().copied().enumerate() {
        let launch = submit_registry_material_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            3_801 + index as u64,
            &cache_roots.runtime_root,
            &cache_roots.staged_root,
        );
        assert_registry_product_shader_cache_hit(&launch, case);
    }

    let _ = fs::remove_dir_all(&cache_roots.root);
}

fn submit_registry_material_with_staged_cache(
    case: RegistryShaderCase,
    shader_source: &str,
    world: u64,
    runtime_root: &Path,
    staged_root: &Path,
) -> RenderStats {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    register_registry_shader(&asset_manager, case, shader_source);
    register_registry_material(&asset_manager, case);

    let framework = WgpuRenderFramework::new(asset_manager).expect("WGPU framework");
    framework.replace_shader_variant_disk_cache_for_tests(
        ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root]),
    );
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .expect("viewport");
    let pipeline = framework
        .register_pipeline_asset(registry_product_pipeline())
        .expect("project/plugin registry product pipeline");
    framework
        .set_pipeline_asset(viewport, pipeline)
        .expect("set registry product pipeline");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("project-plugin-registry-staged-cache")
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_anti_alias(false),
        )
        .expect("quality profile");

    let mut extract = static_cache_extract(case.material_id(), world);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    framework
        .submit_frame_extract(viewport, extract)
        .expect("submit project/plugin registry staged-cache product extract");
    framework.query_stats().expect("render stats")
}

fn register_registry_shader(
    asset_manager: &ProjectAssetManager,
    case: RegistryShaderCase,
    source: &str,
) {
    let shader_uri = case.shader_uri();
    let shader_id = ResourceId::from_locator(&shader_uri);
    let source_hash = raw_wgsl_hash(source);
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone())
                .with_source_hash(source_hash.as_str())
                .with_importer_id("zircon.plan08.registry-product")
                .with_importer_version(1)
                .with_config_hash(source_hash.as_str()),
            ShaderAsset {
                uri: shader_uri.clone(),
                kind: ShaderAssetKind::Surface,
                source_language: ShaderSourceLanguage::Wgsl,
                source: source.to_string(),
                wgsl_source: String::new(),
                import_path: None,
                entry_points: Vec::new(),
                dependencies: Vec::new(),
                source_files: Vec::new(),
                imports: Vec::new(),
                shader_defs: Vec::new(),
                property_schema: Vec::new(),
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: Some("standard_pbr".to_string()),
                render_state: Default::default(),
                queue: None,
                disabled_passes: Vec::new(),
                resources: Vec::new(),
                material_property_layout: Default::default(),
                material_option_table: Default::default(),
                generated_material_wgsl: String::new(),
                editor: Default::default(),
                pipeline_layout: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("registry shader insert");

    let mut exported_record = ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri)
        .with_source_hash(source_hash.as_str())
        .with_importer_id("zircon.plan08.registry-product")
        .with_importer_version(1)
        .with_config_hash(source_hash);
    exported_record.revision = case.revision;
    exported_record.state = ResourceState::Ready;
    asset_manager
        .resource_manager()
        .register_record(exported_record);
}

fn register_registry_material(asset_manager: &ProjectAssetManager, case: RegistryShaderCase) {
    let mut material = material_with_import_note();
    material.name = Some(format!("Plan08RegistryMaterial{}", case.revision));
    material.shader = AssetReference::from_locator(case.shader_uri());
    material.validation_diagnostics.clear();
    register_material_asset_revision(
        asset_manager,
        case.material_id(),
        case.material_uri(),
        "project-plugin-registry-material-v1",
        material,
    );
}

fn registry_product_prewarm_manifest(cases: &[RegistryShaderCase]) -> ShaderVariantPrewarmManifest {
    let forward_request = builtin_standard_material_shader_prewarm_manifest_for_geometry(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_STANDARD_PBR,
        None,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        &[ShaderQualityTier::Medium],
    )
    .variants
    .into_iter()
    .find(|request| request.key.pass_type == ShaderPassType::Forward)
    .expect("builtin forward shader prewarm request");
    let variants = cases
        .iter()
        .map(|case| {
            let mut request = forward_request.clone();
            request.key.material_shader = case.shader_id();
            request.key.material_revision = case.revision;
            request.source_label = case.locator.to_string();
            request
        })
        .collect();
    ShaderVariantPrewarmManifest::new(variants)
}

fn assert_registry_product_prewarm_written(
    manifest: &ShaderVariantPrewarmManifest,
    report: &ShaderVariantPrewarmReport,
    case: RegistryShaderCase,
) {
    let request = manifest
        .variants
        .iter()
        .find(|request| request.source_label == case.locator)
        .unwrap_or_else(|| panic!("registry prewarm request for {}", case.locator));
    assert_registry_request_key(request, case);
    let written = report
        .written_variants
        .iter()
        .find(|variant| variant.source_label == case.locator)
        .unwrap_or_else(|| panic!("registry written variant for {}", case.locator));
    assert!(
        written
            .canonical_string
            .contains(&case.shader_id().to_string()),
        "written cache key should include registry shader id for {}; canonical={}",
        case.locator,
        written.canonical_string
    );
    assert!(
        written
            .canonical_string
            .contains(&format!("|revision={}", case.revision)),
        "written cache key should include registry revision for {}; canonical={}",
        case.locator,
        written.canonical_string
    );
}

fn assert_registry_product_shader_cache_hit(stats: &RenderStats, case: RegistryShaderCase) {
    assert!(
        stats.last_mesh_opaque_draw_count >= 1,
        "product submit should draw the registry material mesh for {}; stats={stats:?}",
        case.locator
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executor| executor == "mesh.opaque"),
        "product submit should execute mesh.opaque for {}; executed={:?}",
        case.locator,
        stats.last_graph_executed_executor_ids
    );
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        report.request_count >= 1,
        "runtime should request registry shader variant for {}; stats={stats:?}",
        case.locator
    );
    assert!(
        report.disk_hit_count >= 1,
        "runtime should disk-hit staged registry shader variant for {}; report={report:?}",
        case.locator
    );
    assert_eq!(
        report.compile_miss_count, 0,
        "runtime must not compile-miss product registry shader variant for {}; report={report:?}",
        case.locator
    );
    assert_eq!(report.disk_write_count, 0);
    assert_eq!(report.disk_error_count, 0);
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.pass_types.get("forward"),
        case,
        "forward pass",
    );
    assert_runtime_dimension_disk_hit(
        report
            .dimension_summary
            .geometry_source_ids
            .get(&GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string()),
        case,
        "static geometry source",
    );
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.quality_tiers.get("medium"),
        case,
        "medium quality tier",
    );
}

fn assert_registry_request_key(request: &ShaderVariantPrewarmRequest, case: RegistryShaderCase) {
    assert_eq!(request.key.material_shader, case.shader_id());
    assert_eq!(request.key.material_revision, case.revision);
    assert_eq!(request.source_label, case.locator);
    let request_source_hash = raw_wgsl_hash(&request.wgsl_source);
    assert!(
        request
            .include_content_hashes
            .contains(&request_source_hash),
        "registry product prewarm request should preserve the assembled source hash for {}; hashes={:?}",
        case.locator,
        request.include_content_hashes
    );
    assert_eq!(request.template_revision, MATERIAL_SHADER_TEMPLATE_REVISION);
}

fn assert_runtime_dimension_disk_hit(
    count: Option<&crate::core::framework::render::ShaderVariantRuntimeDimensionCount>,
    case: RegistryShaderCase,
    dimension_label: &str,
) {
    let count = count.unwrap_or_else(|| {
        panic!(
            "runtime should report {dimension_label} dimension for {}",
            case.locator
        )
    });
    assert!(
        count.disk_hit_count >= 1,
        "runtime should disk-hit staged cache for {dimension_label} on {}; count={count:?}",
        case.locator
    );
    assert_eq!(count.compile_miss_count, 0);
}

fn registry_product_pipeline() -> RenderPipelineAsset {
    RenderPipelineAsset {
        handle: RenderPipelineHandle::new(810),
        revision: 1,
        name: "plan08-project-plugin-registry-staged-cache-product".to_string(),
        core_pipeline: CorePipelineKind::Core3d,
        phase_mapping: vec![RenderPhase::Prepass, RenderPhase::Opaque3d],
        renderer: RendererAsset {
            name: "plan08-project-plugin-registry-staged-cache-renderer".to_string(),
            stages: vec![RenderPassStage::DepthPrepass, RenderPassStage::Opaque3d],
            features: vec![RendererFeatureAsset::plugin(registry_product_feature())],
        },
    }
}

fn registry_product_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.project_plugin_registry_staged_cache_product",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plan08-project-plugin-registry-preview-clear",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-scene-color")
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque3d,
                "plan08-project-plugin-registry-opaque-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.opaque")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

fn raw_wgsl_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn registry_shader_cases() -> [RegistryShaderCase; 2] {
    [
        RegistryShaderCase {
            locator: "res://project/shaders/project_shader",
            material_locator: "res://materials/project_registry_staged_cache.zmaterial",
            revision: 126_198_881_308_539_824,
        },
        RegistryShaderCase {
            locator: "package://native_dynamic_fixture/shaders/shader",
            material_locator: "res://materials/plugin_registry_staged_cache.zmaterial",
            revision: 14_843_875_089_575_827_114,
        },
    ]
}

#[derive(Clone, Copy)]
struct RegistryShaderCase {
    locator: &'static str,
    material_locator: &'static str,
    revision: u64,
}

impl RegistryShaderCase {
    fn shader_uri(self) -> AssetUri {
        AssetUri::parse(self.locator).expect("registry shader URI")
    }

    fn material_uri(self) -> AssetUri {
        AssetUri::parse(self.material_locator).expect("registry material URI")
    }

    fn shader_id(self) -> ResourceId {
        ResourceId::from_locator(&self.shader_uri())
    }

    fn material_id(self) -> ResourceId {
        ResourceId::from_locator(&self.material_uri())
    }
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
