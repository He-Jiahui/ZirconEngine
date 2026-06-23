use super::*;

#[test]
fn runtime_15_shader_prewarm_manifest_tests_are_folder_backed() {
    let parent = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "shader prewarm manifest parent mounts test child",
        &parent,
        &[
            "#[cfg(test)]\nmod tests;",
            "pub fn read_manifest",
            "pub fn asset_root_manifest_for_quality_tiers",
            "fn content_hash",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "bin/zircon_shader_prewarm/manifest.rs should mount child tests instead of defining executable tests"
    );
    assert!(
        !parent.contains("fn shader_prewarm_asset_root_manifest_reads_compound_zshader_package"),
        "bin/zircon_shader_prewarm/manifest.rs should mount the asset-root manifest test owner"
    );
    assert_contains_all(
        "shader prewarm manifest test child owns asset-root manifest contract",
        &tests,
        &[
            "use super::asset_root_manifest;",
            "fn shader_prewarm_asset_root_manifest_reads_compound_zshader_package",
            "super::ASSET_SCAN_INITIAL_RESOURCE_REVISION",
            "SHADING_MODEL_ID_BLINN_PHONG",
            "SHADING_MODEL_ID_UNLIT",
        ],
    );
    assert_eq!(
        tests.matches("#[test]").count(),
        1,
        "shader prewarm manifest child should preserve the single asset-root manifest test"
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/manifest.rs", parent.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 shader prewarm manifest test folder split",
                "runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred",
                "bin/zircon_shader_prewarm/manifest.rs",
                "bin/zircon_shader_prewarm/manifest/tests.rs",
                "runtime_15_shader_prewarm_manifest_tests_are_folder_backed",
            ],
        );
    }
}
