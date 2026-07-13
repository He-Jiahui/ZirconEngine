use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_submit_profile_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/render_product_submit.rs");
    let profiles = read_runtime_src("graphics/tests/render_product_submit/profiles.rs");

    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit_doc =
        read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "render product submit parent keeps base submit/material coverage and child mount",
        &parent,
        &[
            "mod profiles;",
            "fn render_product_submit_direct_extract_frame_does_not_use_legacy_scene_snapshot_authority(",
            "fn render_product_submit_unknown_viewport_returns_error_without_panic(",
            "fn render_product_submit_selects_default_pipeline_from_extract_core_pipeline(",
            "fn render_product_submit_preserves_quality_profile_pipeline_override(",
            "fn render_product_pbr_submit_reports_material_fallback_and_light_stats(",
            "fn render_product_submit_material_stats_count_non_blocking_diagnostics(",
            "fn render_product_submit_material_stats_count_material_uniform_diagnostics(",
            "pub(super) fn snapshot_with_projection_for_sprite_tests(",
            "pub(super) fn snapshot_with_projection_for_mesh_cache_tests(",
        ],
    );

    for moved_anchor in [
        "fn render_product_submit_default_profile_accepts_default_3d_ui_and_2d_sprite_paths(",
        "fn render_product_submit_headless_profile_has_no_render_product_activation(",
        "fn render_product_submit_advanced_profile_accepts_provider_backed_vg_hgi_path(",
        "fn render_product_submit_solari_experimental_reports_gated_provider_status(",
        "fn default_core3d_acceptance_extract(",
        "fn default_core2d_sprite_acceptance_extract(",
        "fn runtime_ui_acceptance_extract(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_submit.rs should delegate `{moved_anchor}` to render_product_submit/profiles.rs"
        );
        assert!(
            profiles.contains(moved_anchor),
            "render_product_submit/profiles.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "profile child keeps default/headless/advanced/solari acceptance details",
        &profiles,
        &[
            "RenderProductProfile::DefaultRender",
            "RenderProductProfile::Headless",
            "RenderProductProfile::AdvancedRender",
            "RenderProductProfile::SolariExperimental",
            "render_product_advanced::advanced_quality_profile",
            "render_product_solari::solari_quality_profile",
            "CorePipelineKind::Core2d",
            "UiRenderCommandKind::Image",
        ],
    );

    for (path, source) in [
        ("graphics/tests/render_product_submit.rs", parent.as_str()),
        (
            "graphics/tests/render_product_submit/profiles.rs",
            profiles.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product submit test budget after the profile split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        (
            "render product submit doc",
            render_product_submit_doc.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product submit profile tests owner split",
                "render_plan09_product_submit_profile_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/render_product_submit.rs",
                "graphics/tests/render_product_submit/profiles.rs",
                "runtime_15_render_product_submit_profile_tests_are_child_owner",
            ],
        );
    }
}
