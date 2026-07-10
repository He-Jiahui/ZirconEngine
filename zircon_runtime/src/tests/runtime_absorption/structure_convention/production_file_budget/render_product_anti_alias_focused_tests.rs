use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_anti_alias_focused_tests_are_child_owners() {
    let parent = read_runtime_src("graphics/tests/render_product_anti_alias.rs");
    let particle = read_runtime_src("graphics/tests/render_product_anti_alias/particle.rs");
    let reactive_mask =
        read_runtime_src("graphics/tests/render_product_anti_alias/reactive_mask.rs");

    let plan_07 = read_repo("docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let anti_alias_doc = read_repo("docs/zircon_runtime/core/framework/render/anti_alias.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "anti-alias product parent keeps general AA/TAA product tests, shared fixtures, and child mounts",
        &parent,
        &[
            "mod particle;",
            "mod reactive_mask;",
            "fn anti_alias_settings_report_structured_fallbacks(",
            "fn render_product_anti_alias_compiles_fxaa_pass_for_default_3d(",
            "fn render_product_temporal_off_matches_anti_alias_feature_disabled_product(",
            "fn render_product_taa_uses_temporal_resolve_seed_frame_when_requested(",
            "fn render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame(",
            "fn render_product_taa_dynamic_occlusion_change_converges_after_history_seed(",
            "fn submit_and_capture_anti_alias_product(",
            "fn frame_rgba_abs_delta(",
        ],
    );

    for moved_anchor in [
        "fn render_product_taa_particle_transparent_pass_contributes_before_resolve(",
        "fn render_product_particle_previous_state_suppresses_velocity_gap_stats(",
        "fn particle_taa_product_extract(",
        "fn particle_motion_blur_taa_product_extract(",
        "fn particle_transparent_billboard_executor(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_anti_alias.rs should delegate `{moved_anchor}` to particle.rs"
        );
        assert!(
            particle.contains(moved_anchor),
            "anti-alias particle child owner should contain `{moved_anchor}`"
        );
    }

    for moved_anchor in [
        "fn render_product_taa_authored_reactive_mask_records_material_writer_path(",
        "fn render_product_taa_transparent_reactive_mask_records_alpha_writer_path(",
        "fn register_taa_reactive_product_material(",
        "fn authored_reactive_mask_taa_product_extract(",
        "fn assert_taa_reactive_mask_graph_executed(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_anti_alias.rs should delegate `{moved_anchor}` to reactive_mask.rs"
        );
        assert!(
            reactive_mask.contains(moved_anchor),
            "anti-alias reactive-mask child owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "anti-alias particle child keeps plugin descriptor, particle snapshots, and executor fixture",
        &particle,
        &[
            "use super::{",
            "particle_render_feature_descriptor",
            "RenderParticleSpriteSnapshot",
            "RenderParticlePreviousSpriteSnapshot",
            "RenderPassExecutorRegistration",
            "record_particle_billboards_to_resources",
        ],
    );

    assert_contains_all(
        "anti-alias reactive-mask child keeps material registration, explicit geometry phases, and TAA executor assertions",
        &reactive_mask,
        &[
            "use super::{",
            "register_taa_reactive_product_material",
            "taa_reactive_mask_strength",
            "GeometryPhaseInput::new",
            "RenderMaterialAlphaMode::Blend",
            "TAA_REACTIVE_MASK_CLEAR_EXECUTOR_ID",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_anti_alias.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_anti_alias/particle.rs",
            particle.as_str(),
        ),
        (
            "graphics/tests/render_product_anti_alias/reactive_mask.rs",
            reactive_mask.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 07", plan_07.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("anti-alias docs", anti_alias_doc.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product anti-alias particle/reactive tests owner split",
                "render_plan07_product_anti_alias_particle_reactive_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/render_product_anti_alias.rs",
                "graphics/tests/render_product_anti_alias/particle.rs",
                "graphics/tests/render_product_anti_alias/reactive_mask.rs",
                "runtime_15_render_product_anti_alias_focused_tests_are_child_owners",
            ],
        );
    }
}
