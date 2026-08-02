use super::*;

const STATUS: &str =
    "render_plan08_shader_prewarm_source_provenance_summary_python_passed_cargo_deferred";
const PYTHON_TEST: &str = "test_dimension_summary_lines_format_source_provenance";

#[test]
fn runtime_15_shader_prewarm_source_provenance_summary_is_wired() {
    let variant_prewarm = read_runtime_src("core/framework/render/shader/variant_prewarm.rs");
    let shader_mod = read_runtime_src("core/framework/render/shader/mod.rs");
    let render_mod = read_runtime_src("core/framework/render/mod.rs");
    let prewarm = read_runtime_src("graphics/shader/variant_cache/prewarm.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let manifest_tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let report_contract = read_repo("tools/zircon_build_shader_prewarm_report_contract.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "prewarm DTO carries request source labels and report provenance",
        &variant_prewarm,
        &[
            "pub source_label: String",
            "pub source_provenance: ShaderVariantPrewarmSourceProvenanceSummary",
            "pub struct ShaderVariantPrewarmSourceProvenanceSummary",
            "pub struct ShaderVariantPrewarmSourceProvenanceEntry",
            "record_written_request",
            "record_failure_request",
            "shader_prewarm_source_hash",
            "shader_prewarm_provenance_key",
        ],
    );
    assert_contains_all(
        "render facades re-export provenance DTOs",
        &format!("{shader_mod}\n{render_mod}"),
        &[
            "ShaderVariantPrewarmSourceProvenanceEntry",
            "ShaderVariantPrewarmSourceProvenanceSummary",
        ],
    );
    assert_contains_all(
        "prewarm write path records request-level provenance",
        &prewarm,
        &[
            "report.record_written_cache_entry(",
            "report.record_failure_request(",
            "report.source_provenance.source_count",
            "written source provenance",
            "failed source provenance",
        ],
    );
    assert_contains_all(
        "manifest producers assign human-readable source labels",
        &format!("{manifest}\n{manifest_tests}\n{dynamic_api}"),
        &[
            "ShaderVariantPrewarmSource::new(",
            "BUILTIN_STANDARD_MATERIAL_SOURCE_LABEL",
            "builtin://shader/pbr.wgsl",
            "source.source_label",
        ],
    );
    assert_contains_all(
        "build helper formats source provenance summaries",
        &report_contract,
        &[
            "def _format_source_provenance(provenance: object) -> str | None:",
            "source_provenance",
            "source_hash=",
            "template=",
        ],
    );
    assert_contains_all(
        "python regression covers provenance-only report output",
        &build_prewarm_tests,
        &[PYTHON_TEST, "source provenance:", "res://shaders/example"],
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
            "zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs",
            manifest.as_str(),
        ),
        (
            "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs",
            manifest_tests.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm_report_contract.py",
            report_contract.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs",
            include_str!("shader_prewarm_source_provenance_summary.rs"),
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
                "Shader prewarm source provenance summary",
                STATUS,
                PYTHON_TEST,
                "runtime_15_shader_prewarm_source_provenance_summary_is_wired",
            ],
        );
    }
}
