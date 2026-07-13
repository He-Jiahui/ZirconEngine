use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_material_product_debug_counts_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/scene/render_product_material_property_tests.rs");
    let debug_counts = read_runtime_src(
        "graphics/scene/render_product_material_property_tests/uniform_debug_counts.rs",
    );

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_assets_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let zmeta_material_doc = read_repo("docs/zircon_runtime/asset/zmeta-shader-material.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");

    assert_contains_all(
        "material product parent keeps compact product tests, fixtures, and child mount",
        &parent,
        &[
            "mod uniform_debug_counts;",
            "fn render_product_material_properties_prepare_uniform_payload(",
            "fn render_product_streamer_reuses_material_uniforms_for_unchanged_revision(",
            "fn render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report(",
            "fn render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults(",
            "fn material_with_shader(",
            "fn shader_with_property_schema(",
            "fn texture_bind_group_layout(",
        ],
    );

    assert!(
        !parent.contains("fn render_product_streamer_exposes_material_uniform_debug_counts("),
        "render_product_material_property_tests.rs should delegate debug-counts coverage to the child owner"
    );

    assert_contains_all(
        "material product debug-counts child keeps the long aggregate coverage",
        &debug_counts,
        &[
            "use super::*;",
            "fn render_product_streamer_exposes_material_uniform_debug_counts(",
            "material_management_record_set",
            "material_management_query_selection",
            "material_management_selection(",
            "[material_id, missing_material_id, material_id]",
            "value_summary.uniform_eligible_count()",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/render_product_material_property_tests.rs",
            parent.as_str(),
        ),
        (
            "graphics/scene/render_product_material_property_tests/uniform_debug_counts.rs",
            debug_counts.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render asset docs", render_assets_doc.as_str()),
        ("zmeta material docs", zmeta_material_doc.as_str()),
        ("material docs", material_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render material product debug-counts test owner split",
                "render_plan08_material_product_debug_counts_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/render_product_material_property_tests.rs",
                "graphics/scene/render_product_material_property_tests/uniform_debug_counts.rs",
                "runtime_15_render_material_product_debug_counts_tests_are_child_owner",
            ],
        );
    }
}
