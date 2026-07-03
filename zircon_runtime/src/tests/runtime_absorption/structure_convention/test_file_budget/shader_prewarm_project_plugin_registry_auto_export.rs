use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_auto_export_nonempty_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_auto_export_is_wired() {
    let acceptance = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let registry = read_repo("tools/zircon_build_shader_resource_registry.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "acceptance forwards project/plugin auto-export nonempty requirement",
        &acceptance,
        &[
            "require_usable_shader_records=",
            "_requires_project_plugin_registry_auto_export",
            "shader_asset_roots",
            "plugin, \"asset_roots\"",
        ],
    );
    assert_contains_all(
        "registry export contract can require usable Shader ResourceRecords",
        &registry,
        &[
            "require_usable_shader_records: bool = False",
            "_validate_registry_export_has_usable_shader_records",
            "_is_usable_shader_record",
            "usable Shader ResourceRecord",
        ],
    );
    assert_contains_all(
        "python tests cover project/plugin auto-export nonempty contract",
        &acceptance_tests,
        &[
            "test_acceptance_contract_requires_usable_records_for_project_plugin_auto_export",
            "asset_roots=(Path(\"plugins\") / \"toon\" / \"assets\",)",
            "usable Shader ResourceRecord",
        ],
    );
    assert_contains_all(
        "registry tests cover optional usable-record requirement",
        &registry_tests,
        &[
            "test_validate_registry_export_contract_requires_usable_shader_records_when_requested",
            "test_validate_registry_export_contract_accepts_usable_shader_records_when_requested",
            "require_usable_shader_records=True",
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
            "tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py",
            registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_auto_export.rs",
            include_str!("shader_prewarm_project_plugin_registry_auto_export.rs"),
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
                "Project/plugin registry auto-export nonempty acceptance",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_auto_export_is_wired",
                "test_acceptance_contract_requires_usable_records_for_project_plugin_auto_export",
            ],
        );
    }
}
