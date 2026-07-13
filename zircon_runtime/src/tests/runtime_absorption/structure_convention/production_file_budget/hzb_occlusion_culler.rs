use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_hzb_occlusion_culler_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs");

    let plan_04 = read_repo(
        "docs/plans/zircon_runtime/render/04/2026-07-09-visibility-culling-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");

    assert_contains_all(
        "HZB culler parent keeps production owner and test mount",
        &parent,
        &[
            "pub(crate) struct HzbOcclusionCuller",
            "pub(crate) fn execute(",
            "fn create_hzb_occlusion_bind_group_layout(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn hzb_occlusion_limit_gate_requires_pipeline_storage_buffer_capacity(",
        "fn hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu(",
        "fn hzb_occlusion_culler_shader_declares_expected_bindings(",
        "fn hzb_occlusion_gpu_stats_remains_copy_aligned(",
        "fn hzb_occlusion_uploads_phase_params_in_encoder_order(",
        "fn hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "HZB culler parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "HZB culler test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "HZB culler test owner keeps WGPU fixtures and parent source scans",
        &tests,
        &[
            "use super::*;",
            "RenderBackend::new_offscreen()",
            "include_str!(\"../hzb_occlusion_culler.rs\")",
            "fn test_hzb_texture(",
            "fn collect_indirect_args_snapshot(",
        ],
    );

    for (path, source) in [
        (
            "scene_renderer/hzb/hzb_occlusion_culler.rs",
            parent.as_str(),
        ),
        (
            "scene_renderer/hzb/hzb_occlusion_culler/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 04", &plan_04),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("visibility docs", &visibility_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "HZB occlusion culler test owner split",
                "render_plan04_hzb_occlusion_culler_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs",
                "graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs",
                "runtime_15_hzb_occlusion_culler_tests_are_child_owner",
            ],
        );
    }
}
