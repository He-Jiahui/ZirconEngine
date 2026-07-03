use super::*;

const STATUS: &str =
    "render_plan08_build_tool_resource_registry_export_contract_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_resource_registry_export_contract_is_wired() {
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let resource_registry = read_repo("tools/zircon_build_shader_resource_registry.py");
    let registry_contract_sources = format!("{build_prewarm}\n{resource_registry}");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py");
    let registry_export_test_sources =
        format!("{build_prewarm_tests}\n{acceptance_tests}\n{registry_tests}");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build helper validates auto-exported shader resource registry shape",
        &registry_contract_sources,
        &[
            "def validate_shader_resource_registry_export_contract(",
            "def _resource_registry_record_array(",
            "shader prewarm resource registry export did not produce",
            "a ResourceRecord array",
            "registry.get(\"resources\")",
            "registry.get(\"records\")",
            "non-object ResourceRecord entries",
        ],
    );
    assert_contains_all(
        "python regressions cover registry export contract enforcement",
        &registry_export_test_sources,
        &[
            "test_validate_registry_export_contract_requires_resource_records",
            "test_validate_registry_export_contract_accepts_wrapped_resources",
            "test_validate_registry_export_contract_accepts_raw_array",
            "registry:{config.shader_prewarm_resource_registry_path}",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/zircon_build_shader_resource_registry.py",
            resource_registry.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py",
            acceptance_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py",
            registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs",
            include_str!("shader_prewarm_resource_registry_export_contract.rs"),
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
                "Build-tool shader resource registry export contract",
                STATUS,
                "test_validate_registry_export_contract_requires_resource_records",
                "runtime_15_shader_prewarm_resource_registry_export_contract_is_wired",
            ],
        );
    }
}
