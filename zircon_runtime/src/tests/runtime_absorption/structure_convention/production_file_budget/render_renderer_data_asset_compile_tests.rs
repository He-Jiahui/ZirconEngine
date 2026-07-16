use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_renderer_data_asset_compile_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/renderer_data_asset.rs");
    let compile_tests =
        read_runtime_src("graphics/tests/renderer_data_asset/asset_aware_compile.rs");

    let plan_08 = read_repo("docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let srp_renderer_data_doc = read_repo("docs/zircon_runtime/graphics/srp-renderer-data.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "renderer-data parent keeps document projection tests, shared fixtures, and asset-aware child mount",
        &parent,
        &[
            "mod asset_aware_compile;",
            "fn renderer_data_document_toml_roundtrip_preserves_srp_fields(",
            "fn renderer_data_document_converts_to_renderer_asset(",
            "fn renderer_data_document_uses_builtin_feature_authoring_name_contract(",
            "fn renderer_data_document_rejects_unknown_feature_sources(",
            "fn asset_reference(",
            "struct InMemoryRenderPipelineAssetContext",
            "fn shader_contract(",
            "fn assert_material_validation(",
        ],
    );

    for moved_anchor in [
        "fn asset_aware_compile_reports_missing_shader_and_material_without_blocking_graph(",
        "fn asset_aware_compile_reports_shader_contract_expectation_gaps(",
        "fn asset_aware_compile_reports_shader_payload_readiness_gaps(",
        "fn asset_aware_compile_reports_material_contract_diagnostics(",
        "fn asset_aware_compile_reports_material_local_validation_diagnostics(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "renderer_data_asset.rs should delegate `{moved_anchor}` to asset_aware_compile.rs"
        );
        assert!(
            compile_tests.contains(moved_anchor),
            "renderer-data asset-aware compile child should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "asset-aware compile child keeps compile options, diagnostics, and shared parent fixtures",
        &compile_tests,
        &[
            "use super::{",
            "RenderPipelineCompileOptions",
            "RendererFeatureContractDiagnostic",
            "RenderShaderDefinitionValue",
            "material_with_contract_gaps",
            "assert_material_validation",
        ],
    );

    for (path, source) in [
        ("graphics/tests/renderer_data_asset.rs", parent.as_str()),
        (
            "graphics/tests/renderer_data_asset/asset_aware_compile.rs",
            compile_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 renderer-data test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("SRP RendererData docs", srp_renderer_data_doc.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "RendererData asset-aware compile tests owner split",
                "render_plan08_renderer_data_asset_compile_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/renderer_data_asset.rs",
                "graphics/tests/renderer_data_asset/asset_aware_compile.rs",
                "runtime_15_renderer_data_asset_compile_tests_are_child_owner",
            ],
        );
    }
}
