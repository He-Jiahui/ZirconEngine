use super::*;

const STATUS: &str =
    "render_plan08_prewarm_wgpu_module_validation_gate_python_cargo_check_passed_runtime_run_timeout_deferred";

#[test]
fn runtime_15_shader_prewarm_wgpu_module_validation_is_wired() {
    let prewarm = read_runtime_src("graphics/shader/variant_cache/prewarm.rs");
    let shader_mod = read_runtime_src("graphics/shader/mod.rs");
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let dynamic_api_mod = read_runtime_src("dynamic_api/mod.rs");
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "shader cache prewarm validates WGPU modules before disk writes",
        &prewarm,
        &[
            "prewarm_shader_variants_to_disk_with_module_validation",
            "validate_shader_variant_prewarm_wgsl",
            "WGPU shader module validation failed",
            "continue;",
            "render_shader_variant_prewarm_rejects_wgpu_module_validation_failure_before_disk_write",
        ],
    );
    assert_contains_all(
        "graphics shader module exposes the validation-capable prewarm entry point",
        &shader_mod,
        &["prewarm_shader_variants_to_disk_with_module_validation"],
    );
    assert_contains_all(
        "dynamic API creates an offscreen WGPU shader module validation gate",
        &dynamic_api,
        &[
            "prewarm_shader_variants_with_wgpu_module_validation",
            "RenderBackend::new_offscreen",
            "device.push_error_scope(wgpu::ErrorFilter::Validation)",
            "device.create_shader_module",
            "wgpu::ShaderSource::Wgsl",
            "WGPU shader module validation setup failed",
        ],
    );
    assert_contains_all(
        "dynamic API surface exports the validation prewarm entry point",
        &dynamic_api_mod,
        &["prewarm_shader_variants_with_wgpu_module_validation"],
    );
    assert_contains_all(
        "shader prewarm CLI parses the opt-in WGPU module validation flag",
        &args,
        &[
            "pub validate_wgpu_modules: bool",
            "\"--validate-wgpu-modules\"",
            "shader_prewarm_args_parse_wgpu_module_validation_flag",
        ],
    );
    assert_contains_all(
        "shader prewarm run path switches only when the WGPU validation flag is set",
        &run,
        &[
            "args.validate_wgpu_modules",
            "prewarm_shader_variants_with_wgpu_module_validation",
            "prewarm_shader_variants(&manifest, &cache_dir)",
        ],
    );
    assert_contains_all(
        "zircon build accepts and stores the opt-in WGPU shader validation flag",
        &build_tool,
        &[
            "--validate-wgpu-shaders",
            "validate_wgpu_shaders: bool",
            "validate_wgpu_shaders=args.validate_wgpu_shaders",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm helper forwards the WGPU module validation flag",
        &build_prewarm,
        &[
            "shader WGPU module validation: enabled",
            "validate_wgpu_shaders",
            "command.append(\"--validate-wgpu-modules\")",
        ],
    );
    assert_contains_all(
        "python command tests cover the WGPU module validation flag",
        &build_prewarm_tests,
        &[
            "test_build_command_forwards_wgpu_shader_module_validation",
            "config.validate_wgpu_shaders = True",
            "\"--validate-wgpu-modules\"",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs",
            prewarm.as_str(),
        ),
        (
            "zircon_runtime/src/dynamic_api/shader_prewarm.rs",
            dynamic_api.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs",
            include_str!("shader_prewarm_wgpu_module_validation.rs"),
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
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Prewarm opt-in WGPU shader-module validation",
                STATUS,
                "test_build_command_forwards_wgpu_shader_module_validation",
                "runtime_15_shader_prewarm_wgpu_module_validation_is_wired",
            ],
        );
    }
}
