use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_pipeline_compile_tests_are_child_owners() {
    let root = read_runtime_src("graphics/pipeline/render_pipeline_asset/compile_tests.rs");
    let core =
        read_runtime_src("graphics/pipeline/render_pipeline_asset/compile_tests/core_contracts.rs");
    let postprocess = read_runtime_src(
        "graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs",
    );
    let external = read_runtime_src(
        "graphics/pipeline/render_pipeline_asset/compile_tests/external_compute_guards.rs",
    );

    let plan_01 = read_repo(
        "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_graph_builder = read_repo("docs/zircon_runtime/render_graph/builder.md");

    assert_contains_all(
        "render pipeline compile test root keeps helper owner and child mounts",
        &root,
        &[
            "mod core_contracts;",
            "mod external_compute_guards;",
            "mod postprocess_routes;",
            "fn test_extract(",
            "fn assert_pass_reads(",
            "fn texture_lifetime<",
        ],
    );

    for moved_test in [
        "fn compile_preserves_renderer_stage_for_each_graph_pass(",
        "fn compile_describes_color_lut_as_rgba16float_3d_transient_when_enabled(",
        "fn compile_preserves_required_external_texture_binding(",
        "fn compile_rejects_compute_workload_on_non_compute_queue(",
    ] {
        assert!(
            !root.contains(moved_test),
            "render pipeline compile test root should not own moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "core compile contract child owns core feature compile tests",
        &core,
        &[
            "use super::*;",
            "fn compile_preserves_renderer_stage_for_each_graph_pass(",
            "fn compile_preserves_compute_workload_from_feature_descriptor(",
            "fn compile_skips_core_particle_pass_when_particle_sprites_miss_selected_camera_layers(",
            "fn compile_describes_hzb_and_ssr_reflection_pyramids_as_mip_chain_transients(",
        ],
    );
    assert_contains_all(
        "postprocess route child owns postprocess compile tests",
        &postprocess,
        &[
            "use super::*;",
            "fn compile_describes_color_lut_as_rgba16float_3d_transient_when_enabled(",
            "fn compile_routes_bloom_extract_after_split_scene_color_passes(",
            "fn compile_routes_output_transfer_through_smaa_terminal_input(",
            "fn compile_describes_hzb_as_half_power_of_two_mip_chain(",
        ],
    );
    assert_contains_all(
        "external compute guard child owns external and compute validation tests",
        &external,
        &[
            "use super::*;",
            "fn compile_preserves_required_external_texture_binding(",
            "fn compile_rejects_conflicting_required_external_texture_and_buffer_binding(",
            "fn compile_rejects_compute_workload_on_non_compute_queue(",
        ],
    );

    for (path, source) in [
        ("render_pipeline_asset/compile_tests.rs", root.as_str()),
        (
            "render_pipeline_asset/compile_tests/core_contracts.rs",
            core.as_str(),
        ),
        (
            "render_pipeline_asset/compile_tests/postprocess_routes.rs",
            postprocess.as_str(),
        ),
        (
            "render_pipeline_asset/compile_tests/external_compute_guards.rs",
            external.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the compile tests split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 01", &plan_01),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render graph builder docs", &render_graph_builder),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "RenderPipelineAsset compile tests owner split",
                "render_pipeline_asset_compile_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/pipeline/render_pipeline_asset/compile_tests.rs",
                "graphics/pipeline/render_pipeline_asset/compile_tests/core_contracts.rs",
                "graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs",
                "graphics/pipeline/render_pipeline_asset/compile_tests/external_compute_guards.rs",
                "runtime_15_render_pipeline_compile_tests_are_child_owners",
            ],
        );
    }
}
