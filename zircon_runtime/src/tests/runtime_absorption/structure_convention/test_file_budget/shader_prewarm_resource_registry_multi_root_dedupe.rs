use super::*;

const STATUS: &str =
    "render_plan08_shader_resource_registry_multi_root_dedupe_static_passed_cargo_deferred";
const TEST: &str = "shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records";

#[test]
fn runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired() {
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let registry = read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let project_record_export = read_runtime_src("asset/project/shader_resource_records.rs");
    let registry_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs");
    let auto_export_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs",
    );
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "shader prewarm run delegates multi-root registry export to resource owner",
        &run,
        &[
            "export_shader_resource_registry_for_asset_roots",
            "shader_resource_records_from_asset_roots(asset_roots)?",
            "serde_json::json!({ \"resources\": records })",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
        ],
    );
    assert!(
        !run.contains("records.extend(shader_resource_records_from_asset_root(asset_root)?)"),
        "run.rs should not own manual multi-root registry concatenation"
    );

    assert_contains_all(
        "resource registry owner delegates multi-root shader records to asset/project",
        &registry,
        &[
            "project_shader_resource_records_from_asset_roots",
            "ShaderResourceRecordExportError",
            "impl From<ShaderResourceRecordExportError> for ShaderPrewarmResourceRegistryError",
        ],
    );
    assert_contains_all(
        "asset/project owner deduplicates multi-root shader records",
        &project_record_export,
        &[
            "shader_resource_records_from_asset_roots",
            "deduplicate_shader_resource_records",
            "records_by_id",
            "ids_by_locator",
            "maps both",
            "left.primary_locator",
            "then_with(|| left.id.cmp(&right.id))",
        ],
    );
    assert_contains_all(
        "resource registry child test covers duplicate shader rows across roots",
        &registry_tests,
        &[
            TEST,
            "shader_resource_records_from_asset_roots_rejects_id_locator_conflicts",
            "shader_resource_records_from_asset_roots_rejects_locator_id_conflicts",
            "engine_assets",
            "plugin_assets",
            "assert_eq!(records.len(), 1)",
            "ResourceLocator::parse(\"res://shaders/shared\")",
        ],
    );
    assert_contains_all(
        "auto-export structure guard keeps the new registry owner in scope",
        &auto_export_guard,
        &[
            "shader_resource_records_from_asset_roots",
            "deduplicate_shader_resource_records",
            TEST,
        ],
    );

    for (path, source) in [
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
            registry.as_str(),
        ),
        (
            "asset/project/shader_resource_records.rs",
            project_record_export.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs",
            registry_tests.as_str(),
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
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Staged shader resource registry multi-root dedupe",
                STATUS,
                "shader_resource_records_from_asset_roots",
                TEST,
                "runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired",
            ],
        );
    }
}
