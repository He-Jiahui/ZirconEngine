use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ssr_gpu_context_tests_are_child_owner_split() {
    let parent = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs",
    );

    let plan_07 = read_repo("docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let post_process_docs = read_repo("docs/zircon_runtime/core/framework/render/post_process.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "SSR GPU context parent keeps production owner and test mount",
        &parent,
        &[
            "fn ssr_parent_pyramid_mip_passes(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_resolve_to_resource(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_reflection_pyramid_coarse_to_resource(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_reflection_pyramid_to_resource(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_reflection_specular_occlusion_to_resource(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn ssr_parent_pyramid_mip_passes_are_empty_for_single_mip_parent(",
        "fn ssr_parent_pyramid_mip_passes_preserve_graph_alias_ops_for_mip_one(",
        "fn ssr_parent_pyramid_mip_passes_clear_later_mips_after_graph_alias_mip(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "SSR GPU context parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "SSR GPU context test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "SSR GPU context test owner keeps private helper coverage",
        &tests,
        &[
            "use super::ssr_parent_pyramid_mip_passes;",
            "RenderGraphAttachmentOps::load_store()",
            "RenderGraphAttachmentOps::clear_store()",
        ],
    );

    for (path, source) in [
        (
            "render_pass_execution_context/gpu/post_process/screen_space_reflection.rs",
            parent.as_str(),
        ),
        (
            "render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the SSR test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 07", &plan_07),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("post-process docs", &post_process_docs),
        ("render product submit docs", &render_product_submit),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "SSR GPU context test owner split",
                "render_plan07_ssr_gpu_context_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs",
                "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs",
                "runtime_15_ssr_gpu_context_tests_are_child_owner_split",
            ],
        );
    }
}
