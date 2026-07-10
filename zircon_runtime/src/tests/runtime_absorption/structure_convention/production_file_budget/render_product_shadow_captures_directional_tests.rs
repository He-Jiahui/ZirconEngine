use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_shadow_captures_directional_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/render_product_shadow_captures.rs");
    let directional =
        read_runtime_src("graphics/tests/render_product_shadow_captures/directional.rs");

    let plan_05 = read_repo("docs/plans/zircon_runtime/render/05/2026-07-09-lighting-shadows-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let shadow_doc = read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");

    assert_contains_all(
        "shadow capture parent keeps spot/multi-spot capture tests, shared material fixtures, and child mount",
        &parent,
        &[
            "mod directional;",
            "fn render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture(",
            "fn render_product_multi_spot_shadow_atlas_darkens_receivers_capture(",
            "fn register_shadow_capture_material(",
            "fn render_spot_shadow_pcf_capture_frame(",
            "fn render_multi_spot_shadow_capture_frame(",
            "fn shadow_capture_settings_with_quality(",
            "fn shadow_capture_mesh(",
            "fn assert_directional_shadow_capture_stats(",
            "fn frame_shadow_darkening_profile(",
        ],
    );

    for moved_anchor in [
        "fn render_product_directional_shadow_atlas_capture_records_receiver_path(",
        "fn render_product_directional_shadow_atlas_darkens_receiver_capture(",
        "fn render_product_csm_directional_remains_stable_under_subtexel_camera_shift(",
        "fn render_product_directional_shadow_atlas_forward_deferred_darkening_parity(",
        "fn render_directional_shadow_capture_frame(",
        "fn render_directional_shadow_capture_frame_with_camera_offset(",
        "fn render_directional_shadow_capture_frame_with_pipeline(",
        "fn directional_shadow_capture_extract(",
        "fn directional_shadow_capture_extract_with_shadow_settings(",
        "fn directional_shadow_capture_settings(",
        "fn assert_darkening_stats_close(",
        "fn assert_darkening_stats_same_product_range(",
        "fn assert_pipeline_executor(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_shadow_captures.rs should delegate `{moved_anchor}` to directional.rs"
        );
        assert!(
            directional.contains(moved_anchor),
            "directional shadow capture child owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "directional shadow capture child keeps directional capture products and shared parent fixture imports",
        &directional,
        &[
            "RenderDirectionalLightSnapshot",
            "RenderPipelineHandle",
            "register_shadow_capture_material",
            "shadow_capture_mesh",
            "directional_shadow_capture_profile",
            "frame_darkened_pixel_count_and_luma_delta",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_shadow_captures.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_shadow_captures/directional.rs",
            directional.as_str(),
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
        ("shadow docs", shadow_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product shadow captures directional test owner split",
                "render_plan05_shadow_capture_directional_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/render_product_shadow_captures.rs",
                "graphics/tests/render_product_shadow_captures/directional.rs",
                "runtime_15_render_product_shadow_captures_directional_tests_are_child_owner",
            ],
        );
    }
}
