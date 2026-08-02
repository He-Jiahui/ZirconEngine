use super::*;

const STATUS: &str = "render_plan08_prewarm_wgpu_render_pipeline_validation_gate_focused_tests_passed_product_deferred";

#[test]
fn runtime_15_shader_prewarm_wgpu_render_pipeline_validation_is_wired() {
    let pipeline_validation = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs",
    );
    let mesh_pipeline_cache_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs");
    let prewarm = read_runtime_src("graphics/shader/variant_cache/prewarm.rs");
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let wgpu_validation = read_runtime_src("dynamic_api/shader_prewarm/wgpu_validation.rs");
    let dynamic_api_mod = read_runtime_src("dynamic_api/mod.rs");
    let resource_limits = read_runtime_src("graphics/resource_limits.rs");
    let request_device = read_runtime_src("graphics/backend/render_backend/request_device.rs");
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let report_contract = read_repo("tools/zircon_build_shader_prewarm_report_contract.py");
    let command_contract_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_command_contract.py");
    let prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let report_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "mesh prewarm validation creates real render pipelines for every material mesh pass",
        &pipeline_validation,
        &[
            "validate_mesh_prewarm_request_render_pipeline",
            "create_mesh_prewarm_validation_pipeline_layout",
            "device.push_error_scope(wgpu::ErrorFilter::Validation)",
            "device.create_shader_module",
            "create_mesh_pipeline",
            "create_gbuffer_mesh_pipeline",
            "create_depth_prepass_mesh_pipeline",
            "create_shadow_mesh_pipeline",
            "create_velocity_mesh_pipeline",
            "create_taa_reactive_mask_mesh_pipeline",
            "mesh_prewarm_pipeline_validation_creates_all_builtin_pass_pipelines",
            "mesh_prewarm_pipeline_validation_rejects_raw_surface_only_wgsl",
        ],
    );
    assert_contains_all(
        "mesh pipeline cache exposes the prewarm validation helpers",
        &mesh_pipeline_cache_mod,
        &[
            "mod prewarm_pipeline_validation;",
            "create_mesh_prewarm_validation_pipeline_layout",
            "validate_mesh_prewarm_request_render_pipeline",
        ],
    );
    assert_contains_all(
        "shader variant cache blocks disk writes on render-pipeline validation failure",
        &prewarm,
        &[
            "prewarm_shader_variants_to_disk_with_pipeline_validation",
            "enable_wgpu_pipeline_validation",
            "record_wgpu_pipeline_validation_passed",
            "record_wgpu_pipeline_validation_failed",
            "WGPU render pipeline validation failed",
            "render_shader_variant_prewarm_rejects_wgpu_pipeline_validation_failure_before_disk_write",
        ],
    );
    assert_contains_all(
        "dynamic API re-exports strict WGPU pipeline prewarm",
        &dynamic_api,
        &[
            "prewarm_shader_variants_with_wgpu_pipeline_validation",
            "pub use wgpu_validation::{",
        ],
    );
    assert_contains_all(
        "WGPU validation owner creates the offscreen handoff for strict pipeline prewarm",
        &wgpu_validation,
        &[
            "RenderBackend::new_offscreen",
            "create_mesh_prewarm_validation_pipeline_layout",
            "validate_mesh_prewarm_request_render_pipeline",
            "WGPU render pipeline validation setup failed",
        ],
    );
    assert_contains_all(
        "render backend requests enough storage-buffer slots for full mesh pipeline validation",
        &resource_limits,
        &[
            "MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 14",
            "OIT_FRAGMENT_STORE_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 2",
            "OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 =",
        ],
    );
    assert_contains_all(
        "offscreen WGPU device limits cover mesh forward pipeline layout requirements",
        &request_device,
        &[
            "MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE",
            "OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE",
            "required_storage_buffers_per_shader_stage",
            ".max(OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE)",
            "offscreen_device_limits_cover_renderer_layout_requirements",
            "offscreen_device_limits_cover_oit_fragment_store_bindings",
            "offscreen_device_limits_keep_hzb_occlusion_optional_when_only_mesh_capacity_exists",
        ],
    );
    assert_contains_all(
        "public dynamic API exports strict pipeline prewarm",
        &dynamic_api_mod,
        &["prewarm_shader_variants_with_wgpu_pipeline_validation"],
    );
    assert_contains_all(
        "shader prewarm CLI parses and prioritizes strict pipeline validation",
        &args,
        &[
            "pub validate_wgpu_pipelines: bool",
            "\"--validate-wgpu-pipelines\"",
            "shader_prewarm_args_parse_wgpu_pipeline_validation_flag",
        ],
    );
    assert_contains_all(
        "shader prewarm runner dispatches strict pipeline validation before module-only validation",
        &run,
        &[
            "args.validate_wgpu_pipelines",
            "prewarm_shader_variants_with_wgpu_pipeline_validation",
            "args.validate_wgpu_modules",
            "prewarm_shader_variants_with_wgpu_module_validation",
        ],
    );
    assert_contains_all(
        "build tool exposes the strict pipeline validation flag",
        &build_tool,
        &[
            "--validate-wgpu-pipelines",
            "validate_wgpu_pipelines: bool",
            "validate_wgpu_pipelines=args.validate_wgpu_pipelines",
        ],
    );
    assert_contains_all(
        "build helper forwards and validates the strict pipeline prewarm flag",
        &build_prewarm,
        &[
            "shader WGPU render pipeline validation: enabled",
            "command.append(\"--validate-wgpu-pipelines\")",
            "WGPU render pipeline validation",
        ],
    );
    assert_contains_all(
        "report contract distinguishes module and render-pipeline validation counts",
        &report_contract,
        &[
            "require_wgpu_pipeline_validation: bool = False",
            "\"wgpu_pipeline_validation\"",
            "\"render pipeline\"",
            "_validate_wgpu_validation_contract(",
            "shader prewarm WGPU {label} validation counts did not match",
        ],
    );
    for (label, source, anchors) in [
        (
            "command contract tests",
            command_contract_tests.as_str(),
            &[
                "test_command_contract_rejects_missing_wgpu_pipeline_validation_flag",
                "\"--validate-wgpu-pipelines\"",
            ][..],
        ),
        (
            "prewarm summary tests",
            prewarm_tests.as_str(),
            &[
                "test_dimension_summary_lines_format_wgpu_pipeline_validation_counts",
                "test_build_command_forwards_wgpu_shader_pipeline_validation",
            ][..],
        ),
        (
            "report contract tests",
            report_tests.as_str(),
            &[
                "test_validate_report_contract_requires_wgpu_pipeline_validation_when_requested",
                "test_validate_report_contract_accepts_wgpu_pipeline_validation_counts",
            ][..],
        ),
        (
            "acceptance contract tests",
            acceptance_tests.as_str(),
            &["test_acceptance_contract_requires_pipeline_validation_when_enabled"][..],
        ),
    ] {
        assert_contains_all(label, source, anchors);
    }

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs",
            pipeline_validation.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs",
            prewarm.as_str(),
        ),
        (
            "zircon_runtime/src/dynamic_api/shader_prewarm.rs",
            dynamic_api.as_str(),
        ),
        (
            "zircon_runtime/src/dynamic_api/shader_prewarm/wgpu_validation.rs",
            wgpu_validation.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm_report_contract.py",
            report_contract.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_pipeline_validation.rs",
            include_str!("shader_prewarm_wgpu_pipeline_validation.rs"),
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
        ("mesh pipeline cache doc", mesh_cache_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Prewarm opt-in WGPU render-pipeline validation",
                STATUS,
                "test_build_command_forwards_wgpu_shader_pipeline_validation",
                "runtime_15_shader_prewarm_wgpu_render_pipeline_validation_is_wired",
            ],
        );
    }
}
