use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_staged_cache_static_passed_cargo_timeout_no_result";
const OWNER_SPLIT_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_owner_split_static_passed_cargo_deferred_active_lanes";
const VELOCITY_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_velocity_runtime_contract_static_passed_cargo_deferred_active_lanes";
const SECOND_LAUNCH_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_second_launch_static_passed_wgpu_timeout_no_result";
const SECOND_LAUNCH_CARGO_WRAPPER_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_second_launch_cargo_wrapper_wgpu_passed_renderdoc_deferred";
const SECOND_LAUNCH_DEFAULT_FEATURE_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_second_launch_default_features_wgpu_passed_renderdoc_deferred";
const CUSTOM_PRODUCT_GROUP_DEFAULT_FEATURE_REFRESH_STATUS: &str =
    "render_plan08_custom_shading_model_product_group_default_features_wgpu_refresh_passed_renderdoc_deferred";
const LIVE_REGISTRY_SOURCE_LABEL_PRODUCT_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_live_registry_source_label_product_wgpu_passed_renderdoc_deferred";
const LIVE_REGISTRY_RECORD_PRODUCT_STATUS: &str =
    "render_plan08_project_plugin_registry_material_passes_asset_root_records_wgpu_passed_renderdoc_deferred";
const SHARED_RESOURCE_RECORD_EXPORT_PRODUCT_STATUS: &str =
    "render_plan08_project_plugin_registry_shared_resource_record_export_product_wgpu_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired() {
    let product_dir =
        "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache";
    assert!(
        !repo_path("zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache.rs").exists(),
        "material-pass registry product owner should stay folder-backed instead of returning to one near-budget file"
    );
    let product_mod = [
        "mod.rs",
        "case.rs",
        "manifest.rs",
        "fixture.rs",
        "pipeline.rs",
        "assertions.rs",
        "custom_shading_model.rs",
        "custom_second_launch.rs",
        "live_registry_bridge.rs",
        "live_registry_records.rs",
        "second_launch.rs",
    ]
    .into_iter()
    .map(|file| read_repo(&format!("{product_dir}/{file}")))
    .collect::<Vec<_>>()
    .join("\n");
    let product_mod_entry = read_repo(&format!("{product_dir}/mod.rs"));
    let product_case = read_repo(&format!("{product_dir}/case.rs"));
    let product_manifest = read_repo(&format!("{product_dir}/manifest.rs"));
    let product_fixture = read_repo(&format!("{product_dir}/fixture.rs"));
    let product_pipeline = read_repo(&format!("{product_dir}/pipeline.rs"));
    let product_assertions = read_repo(&format!("{product_dir}/assertions.rs"));
    let product_custom = read_repo(&format!("{product_dir}/custom_shading_model.rs"));
    let product_custom_second_launch = read_repo(&format!("{product_dir}/custom_second_launch.rs"));
    let product_live_registry = read_repo(&format!("{product_dir}/live_registry_bridge.rs"));
    let product_live_records = read_repo(&format!("{product_dir}/live_registry_records.rs"));
    let product_second_launch = read_repo(&format!("{product_dir}/second_launch.rs"));
    let prewarm_pipeline_validation = read_repo(
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs",
    );
    let project_record_export =
        read_repo("zircon_runtime/src/asset/project/shader_resource_records.rs");
    let parent_mod = read_repo("zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "project/plugin registry material-pass staged-cache product test is wired",
        &product_mod,
        &[
            "render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss",
            "REGISTRY_MATERIAL_PASS_TYPES",
            "res://project/shaders/project_shader",
            "package://native_dynamic_fixture/shaders/shader",
            "builtin_fallback_shader_prewarm_manifest",
            "request.key.material_shader = case.shader_id()",
            "request.key.material_revision = case.revision",
            "request.source_label = case.source_label_for_pass(request.key.pass_type)",
            "ShaderVariantCacheDisk::with_fallback_roots",
            "registry_material_pass_product_pipeline",
            "BuiltinRenderFeature::DeferredGeometry",
            "BuiltinRenderFeature::DeferredLighting",
            "taa_reactive_mask_strength",
            "deferred.depth-prepass",
            "deferred.gbuffer",
            "shadow.atlas",
            "taa_reactive_mask",
            "temporal.velocity-object",
            "last_mesh_previous_velocity_transform_draw_count",
            "last_mesh_missing_velocity_transform_draw_count, 0",
            "report.dimension_summary.pass_types.get(\"velocity\")",
            "velocity pass",
            "report.compile_miss_count, 0",
            "render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss",
            "assert_runtime_shader_cache_root_empty",
            "recursive_file_count",
            "second product launch should still stay read-only against staged cache",
            "registry_material_pass_runtime_surface_source",
            "fn zr_material_surface(",
            "custom_toon_plugin_shading_model",
            "register_registry_shader(",
            "model.descriptor.token.as_str()",
            "shading_model_token.unwrap_or(\"standard_pbr\")",
            "render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss",
            "render_product_custom_shading_model_deferred_lighting_readback_uses_project_include",
            "render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss",
            "render_product_project_plugin_registry_material_passes_live_registry_source_labels_hit_staged_cache",
            "registry_material_pass_live_source_label_prewarm_manifest",
            "request.source_label = case.locator.to_string()",
            "variant.canonical_string == request.key.canonical_string()",
            "live registry source-label product launch should stay read-only against staged cache",
            "render_product_project_plugin_registry_material_passes_asset_root_records_hit_staged_cache",
            "registry_shader_cases_from_live_records",
            "shader_resource_records_from_asset_roots",
            "native_dynamic_fixture_asset_root",
            "project/plugin asset-root shader resource records",
            "asset-root record product launch should stay read-only against staged cache",
            "report={prewarm_report:#?}",
        ],
    );
    assert_contains_all(
        "asset/project record export owns the live ResourceRecord scan rules",
        &project_record_export,
        &[
            "shader_resource_records_from_asset_roots",
            "deduplicate_shader_resource_records",
            "AssetMetaDocument::load",
            "asset_scan_revision_from_source_hash",
            "ResourceKind::Shader",
            "ResourceState::Ready",
        ],
    );
    assert_contains_all(
        "prewarm WGPU material bind-group validation matches runtime material ABI",
        &prewarm_pipeline_validation,
        &[
            "material_uniform_entry(0)",
            "material_texture_entry(1)",
            "material_sampler_entry(2)",
            "GPU_MATERIAL_UNIFORM_MIN_SIZE",
            "visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT",
        ],
    );
    assert_contains_all(
        "render product mesh cache parent declares focused material-pass module",
        &parent_mod,
        &["mod project_plugin_registry_material_passes_staged_cache;"],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs",
            parent_mod.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/mod.rs",
            product_mod_entry.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/case.rs",
            product_case.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/manifest.rs",
            product_manifest.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/fixture.rs",
            product_fixture.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/pipeline.rs",
            product_pipeline.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/assertions.rs",
            product_assertions.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_shading_model.rs",
            product_custom.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_second_launch.rs",
            product_custom_second_launch.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/live_registry_bridge.rs",
            product_live_registry.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/live_registry_records.rs",
            product_live_records.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/second_launch.rs",
            product_second_launch.as_str(),
        ),
        (
            "zircon_runtime/src/asset/project/shader_resource_records.rs",
            project_record_export.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_material_passes_staged_cache.rs",
            include_str!("shader_prewarm_project_plugin_registry_material_passes_staged_cache.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry material-pass staged-cache miss=0",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired",
                "DepthPrepass/GBuffer/Shadow/TAA reactive mask",
                "compile miss=0",
                "Cargo/WGPU execution timed out",
                "Project/plugin registry material-pass owner split",
                OWNER_SPLIT_STATUS,
                "folder-backed owner",
                "old single-file owner removed",
                "Project/plugin registry material-pass Velocity runtime contract",
                VELOCITY_STATUS,
                "temporal.velocity-object",
                "velocity pass",
                "previous velocity transform",
                "Project/plugin registry material-pass second-launch miss=0",
                SECOND_LAUNCH_STATUS,
                "second product launch",
                "Project/plugin registry material-pass second-launch Cargo-wrapper WGPU backfill",
                SECOND_LAUNCH_CARGO_WRAPPER_STATUS,
                "render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss",
                "5842 filtered",
                "5848 filtered",
                "12.86s",
                "Cargo-wrapper",
                "Project/plugin registry material-pass second-launch default-feature WGPU backfill",
                SECOND_LAUNCH_DEFAULT_FEATURE_STATUS,
                "registry_material_pass_runtime_surface_source",
                "fn zr_material_surface(",
                "6177 filtered",
                "6171 filtered",
                "14.96s",
                "12.75s",
                "default-feature",
                "Custom shading-model product group default-feature WGPU refresh",
                CUSTOM_PRODUCT_GROUP_DEFAULT_FEATURE_REFRESH_STATUS,
                "group2 binding0",
                "custom:toon",
                "6187 filtered",
                "6191 filtered",
                "6195 filtered",
                "Project/plugin registry material-pass live registry source-label product bridge",
                LIVE_REGISTRY_SOURCE_LABEL_PRODUCT_STATUS,
                "render_product_project_plugin_registry_material_passes_live_registry_source_labels_hit_staged_cache",
                "live registry source label should not depend on test-only pass suffixes",
                "`PluginShaderModuleManifest` root export",
                "zircon_runtime/src/plugin/mod.rs",
                "6358 filtered",
                "10.44s",
                "RenderDoc/product capture",
                "Project/plugin registry material-pass asset-root ResourceRecord product bridge",
                LIVE_REGISTRY_RECORD_PRODUCT_STATUS,
                "render_product_project_plugin_registry_material_passes_asset_root_records_hit_staged_cache",
                "registry_shader_cases_from_live_records",
                "native_dynamic_fixture/assets/shader.wgsl.zmeta",
                "6373 filtered",
                "10.71s",
                "Project/plugin registry shared ResourceRecord export product bridge",
                SHARED_RESOURCE_RECORD_EXPORT_PRODUCT_STATUS,
                "asset/project/shader_resource_records.rs",
                "project_shader_resource_records_from_asset_roots",
            ],
        );
    }
}
