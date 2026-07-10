use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_shadows_many_point_lights_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/render_product_shadows.rs");
    let many_point_lights =
        read_runtime_src("graphics/tests/render_product_shadows/many_point_lights.rs");

    let plan_05 = read_repo("docs/plans/zircon_runtime/render/05/2026-07-09-lighting-shadows-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "shadow product parent keeps graph, CSM, spot-shadow, shared shadow helpers, and child mount",
        &parent,
        &[
            "mod many_point_lights;",
            "fn shadow_atlas_pass_stays_live_as_depth_only_graph_contract(",
            "fn deferred_lighting_reads_shadow_atlas_for_receiver_sampling(",
            "fn render_product_csm_directional(",
            "fn render_product_multi_spot_shadows(",
            "fn shadow_frame(",
            "fn shadow_settings(",
            "fn pass_resource_access",
        ],
    );

    for moved_anchor in [
        "fn render_product_many_point_lights(",
        "fn render_product_many_point_lights_forward_deferred_capture_parity(",
        "fn render_product_hundred_point_lights_report_local_density_stats(",
        "fn many_point_light_extract(",
        "fn hundred_point_light_density_extract(",
        "fn assert_light_grid_reads(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_shadows.rs should delegate `{moved_anchor}` to many_point_lights.rs"
        );
        assert!(
            many_point_lights.contains(moved_anchor),
            "many-point lights child owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "many-point lights child keeps light-grid product coverage and WGPU capture parity fixtures",
        &many_point_lights,
        &[
            "build_light_grid_for_frame",
            "pack_lighting_extract",
            "GpuLightType::Point",
            "RenderPipelineAsset::default_forward_plus",
            "RenderPipelineAsset::default_deferred",
            "lighting.light-grid",
            "last_light_grid_peak_lights_per_cluster",
        ],
    );

    for (path, source) in [
        ("graphics/tests/render_product_shadows.rs", parent.as_str()),
        (
            "graphics/tests/render_product_shadows/many_point_lights.rs",
            many_point_lights.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 05", plan_05.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product shadows many-point lights test owner split",
                "render_plan05_product_shadows_many_point_lights_test_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/render_product_shadows.rs",
                "graphics/tests/render_product_shadows/many_point_lights.rs",
                "runtime_15_render_product_shadows_many_point_lights_tests_are_child_owner",
            ],
        );
    }
}
