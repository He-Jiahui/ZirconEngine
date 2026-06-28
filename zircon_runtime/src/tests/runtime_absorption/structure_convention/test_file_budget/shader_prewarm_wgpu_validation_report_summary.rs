use super::*;

const STATUS: &str =
    "render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired() {
    let variant_prewarm = read_runtime_src("core/framework/render/shader/variant_prewarm.rs");
    let shader_mod = read_runtime_src("core/framework/render/shader/mod.rs");
    let render_mod = read_runtime_src("core/framework/render/mod.rs");
    let prewarm = read_runtime_src("graphics/shader/variant_cache/prewarm.rs");
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "prewarm report exposes WGPU module validation summary fields",
        &variant_prewarm,
        &[
            "pub wgpu_module_validation: ShaderVariantPrewarmWgpuModuleValidationSummary",
            "pub struct ShaderVariantPrewarmWgpuModuleValidationSummary",
            "pub enabled: bool",
            "pub validated_count: usize",
            "pub skipped_count: usize",
            "enable_wgpu_module_validation",
            "record_wgpu_module_validation_passed",
            "record_wgpu_module_validation_failed",
            "record_wgpu_module_validation_skipped",
        ],
    );
    assert_contains_all(
        "render facade re-exports the report summary DTO",
        &shader_mod,
        &["ShaderVariantPrewarmWgpuModuleValidationSummary"],
    );
    assert_contains_all(
        "core render facade re-exports the report summary DTO",
        &render_mod,
        &["ShaderVariantPrewarmWgpuModuleValidationSummary"],
    );
    assert_contains_all(
        "prewarm write path records WGPU module validation outcomes",
        &prewarm,
        &[
            "report.enable_wgpu_module_validation(manifest.variants.len())",
            "report.record_wgpu_module_validation_skipped()",
            "report.record_wgpu_module_validation_failed()",
            "report.record_wgpu_module_validation_passed()",
            "render_shader_variant_prewarm_records_wgpu_module_validation_success",
        ],
    );
    assert_contains_all(
        "dynamic setup failure path records validation failure counts",
        &dynamic_api,
        &[
            "report.enable_wgpu_module_validation(manifest.variants.len())",
            "report.record_wgpu_module_validation_failed()",
            "WGPU shader module validation setup failed",
        ],
    );
    assert_contains_all(
        "build helper formats WGPU validation and Rust count field summaries",
        &build_prewarm,
        &[
            "def _format_wgpu_module_validation(validation: object) -> str | None:",
            "wgpu_module_validation",
            "_count_value(validation, \"validated\")",
            "_count_value(validation, \"skipped\")",
            "def _count_value(counts: Mapping[str, object], field: str) -> int:",
            "f\"{field}_count\"",
        ],
    );
    assert_contains_all(
        "python tests cover validation summary and Rust count fields",
        &build_prewarm_tests,
        &[
            "test_dimension_summary_lines_accept_rust_count_field_names",
            "test_dimension_summary_lines_format_wgpu_module_validation_counts",
            "WGPU module validation: enabled requested=3",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs",
            variant_prewarm.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs",
            prewarm.as_str(),
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
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs",
            include_str!("shader_prewarm_wgpu_validation_report_summary.rs"),
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
        ("session note", session.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Prewarm WGPU validation report summary",
                STATUS,
                "test_dimension_summary_lines_format_wgpu_module_validation_counts",
                "runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired",
            ],
        );
    }
}
