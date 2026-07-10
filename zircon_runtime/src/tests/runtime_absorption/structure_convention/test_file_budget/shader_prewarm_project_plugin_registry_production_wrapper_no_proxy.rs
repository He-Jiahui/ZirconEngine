use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_production_wrapper_no_proxy_wgpu_passed_product_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired() {
    let build_tool = read_repo("tools/zircon_build.py");
    let wrapper_test =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let build_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "public wrapper separates target-server runtime/prewarm from target-client preview",
        &build_tool,
        &[
            "def runtime_preview_feature_arg",
            "return self.feature_arg_for_target(\"target-client\")",
            "preview_feature_arg = config.runtime_preview_feature_arg",
            "runtime_feature_arg",
            "preview_feature_arg",
        ],
    );
    assert_contains_all(
        "wrapper regression locks runtime lib and preview binary feature split",
        &wrapper_test,
        &[
            "test_runtime_server_wrapper_uses_client_features_for_preview_binary",
            "\"target-server,profiling\"",
            "'cargo-probe build -p zircon_runtime --lib --no-default-features '",
            "'--features \"target-server profiling\"'",
            "'cargo-probe build -p zircon_app --bin zircon_runtime '",
            "'--no-default-features --features \"target-client profiling\"'",
            "test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu",
            "fake_cargo.cmd",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("build tool doc", build_doc.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry production wrapper no-proxy WGPU run",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired",
                "test_runtime_server_wrapper_uses_client_features_for_preview_binary",
                "target-server",
                "target-client",
                "18/18",
                "no-proxy",
            ],
        );
    }

    for (path, source) in [
        (
            "tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py",
            wrapper_test.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }
}
