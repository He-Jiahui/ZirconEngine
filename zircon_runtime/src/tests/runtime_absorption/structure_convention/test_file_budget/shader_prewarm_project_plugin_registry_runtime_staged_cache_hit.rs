use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_runtime_staged_cache_hit_static_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_runtime_staged_cache_hit_is_wired() {
    let ensure_pipeline = read_repo(
        "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "runtime staged-cache hit test uses registry locators as material shader keys",
        &ensure_pipeline,
        &[
            "runtime_project_plugin_registry_shader_keys_use_staged_prewarm_without_compile_miss",
            "RegistryShaderCase",
            "res://project/shaders/project_shader",
            "package://native_dynamic_fixture/shaders/shader",
            "request.key.material_shader = case.resource_id()",
            "request.key.material_revision = case.revision",
            "ShaderVariantCacheDisk::with_fallback_roots",
            "assert_eq!(miss_report.disk_hit_count, registry_cases.len())",
            "assert_eq!(miss_report.compile_miss_count, 0)",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
            ensure_pipeline.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs",
            include_str!("shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs"),
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
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry runtime staged-cache hit",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_runtime_staged_cache_hit_is_wired",
                "compile miss=0",
                "Cargo/WGPU execution deferred",
            ],
        );
    }
}
