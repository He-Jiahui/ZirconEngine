use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_report_source_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_report_source_is_wired() {
    let acceptance = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let registry = read_repo("tools/zircon_build_shader_resource_registry.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let project_plugin_tests = read_repo(
        "tools/tests/test_zircon_build_shader_prewarm_project_plugin_registry_acceptance.py",
    );
    let registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "acceptance requires registry-backed report sources for project/plugin auto-export",
        &acceptance,
        &[
            "require_report_registry_backed_sources=",
            "_requires_project_plugin_registry_auto_export",
            "shader_asset_roots",
            "plugin, \"asset_roots\"",
        ],
    );
    assert_contains_all(
        "registry export contract can require report-visible registry-backed shader sources",
        &registry,
        &[
            "require_report_registry_backed_sources: bool = False",
            "registry-backed report source",
            "requires report_path",
            "_report_resource_source_labels",
        ],
    );
    assert_contains_all(
        "python tests cover project/plugin auto-export source evidence",
        &project_plugin_tests,
        &[
            "test_acceptance_contract_requires_registry_source_for_project_plugin_auto_export",
            "test_acceptance_contract_accepts_registry_source_for_project_plugin_auto_export",
            "registry-backed report source",
        ],
    );
    assert_contains_all(
        "registry tests cover required report source switch",
        &registry_tests,
        &[
            "test_validate_registry_export_contract_requires_report_registry_backed_sources",
            "test_validate_registry_export_contract_accepts_required_report_registry_backed_sources",
            "require_report_registry_backed_sources=True",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm_acceptance.py",
            acceptance.as_str(),
        ),
        (
            "tools/zircon_build_shader_resource_registry.py",
            registry.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py",
            acceptance_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_project_plugin_registry_acceptance.py",
            project_plugin_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py",
            registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_report_source.rs",
            include_str!("shader_prewarm_project_plugin_registry_report_source.rs"),
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
                "Project/plugin registry report-source acceptance",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_report_source_is_wired",
                "test_acceptance_contract_requires_registry_source_for_project_plugin_auto_export",
            ],
        );
    }
}
