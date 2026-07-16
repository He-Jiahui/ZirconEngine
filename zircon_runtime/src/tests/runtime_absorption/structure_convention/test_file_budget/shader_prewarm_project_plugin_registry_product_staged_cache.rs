use super::*;

const STATUS: &str = "render_plan08_project_plugin_registry_product_staged_cache_static_passed_cargo_timeout_no_result";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_product_staged_cache_is_wired() {
    let product_mod = read_repo(
        "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_staged_cache.rs",
    );
    let parent_mod = read_repo("zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "project/plugin registry product staged-cache test is wired",
        &product_mod,
        &[
            "render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss",
            "RegistryShaderCase",
            "res://project/shaders/project_shader",
            "package://native_dynamic_fixture/shaders/shader",
            "register_registry_shader",
            "register_record(exported_record)",
            "request.key.material_shader = case.shader_id()",
            "request.key.material_revision = case.revision",
            "let request_source_hash = raw_wgsl_hash(&request.wgsl_source)",
            ".include_content_hashes",
            ".contains(&request_source_hash)",
            "ShaderVariantCacheDisk::with_fallback_roots",
            "assert_registry_product_shader_cache_hit",
            "report.compile_miss_count, 0",
        ],
    );
    assert_contains_all(
        "render product mesh cache parent declares focused module",
        &parent_mod,
        &["mod project_plugin_registry_staged_cache;"],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs",
            parent_mod.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_staged_cache.rs",
            product_mod.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_product_staged_cache.rs",
            include_str!("shader_prewarm_project_plugin_registry_product_staged_cache.rs"),
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
                "Project/plugin registry product staged-cache miss=0",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_product_staged_cache_is_wired",
                "compile miss=0",
                "Cargo/WGPU execution timed out",
            ],
        );
    }
}
