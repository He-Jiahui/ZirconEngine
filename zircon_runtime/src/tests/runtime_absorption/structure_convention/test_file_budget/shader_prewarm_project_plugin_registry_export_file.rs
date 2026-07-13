use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_export_file_static_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_export_file_is_wired() {
    let run = read_repo("zircon_runtime/src/bin/zircon_shader_prewarm/run.rs");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "run path writes project/plugin registry export file and returns overlay records",
        &run,
        &[
            "shader_prewarm_project_and_plugin_asset_roots_export_wrapped_resource_registry_file",
            "export_shader_resource_registry_for_asset_roots",
            "serde_json::json!({ \"resources\": records })",
            "ZirconEngine",
            "shader_resource_records.json",
            "package://virtual_geometry/shaders/plugin",
            "res://project/shaders/project",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
            "revision_for(record.id, &label)",
        ],
    );

    let line_count = run.lines().count();
    assert!(
        line_count < 800,
        "zircon_shader_prewarm run.rs should stay below the Runtime 15 owner budget; got {line_count} lines"
    );

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
                "Project/plugin registry export file handoff",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_export_file_is_wired",
                "shader_prewarm_project_and_plugin_asset_roots_export_wrapped_resource_registry_file",
            ],
        );
    }
}
