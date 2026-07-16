fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    let missing: Vec<&str> = anchors
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(missing.is_empty(), "{label} missing anchors: {missing:?}");
}

fn assert_not_contains(label: &str, source: &str, anchors: &[&str]) {
    let present: Vec<&str> = anchors
        .iter()
        .copied()
        .filter(|anchor| source.contains(anchor))
        .collect();
    assert!(
        present.is_empty(),
        "{label} unexpected anchors: {present:?}"
    );
}

#[test]
fn review_f16_compiled_scene_render_path_uses_split_owners() {
    let render_mod = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs"
    );
    let render = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs"
    );
    let bind = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs"
    );
    let execute = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs"
    );
    let submit = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs"
    );
    let sprite_stage_selection = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs"
    );
    let pipeline_resource_usage = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/pipeline_resource_usage.rs"
    );
    let review_findings = concat!(
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let render_index = include_str!("../../../../../docs/plans/zircon_runtime/render/index.md");
    let runtime_07 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let framework_doc =
        include_str!("../../../../../docs/assets-and-rendering/render-framework-architecture.md");

    assert_contains_all(
        "render module split",
        render_mod,
        &[
            "mod bind_compiled_scene_graph_resources;",
            "mod execute_compiled_scene_graph_stages;",
            "mod pipeline_resource_usage;",
            "mod sprite_stage_selection;",
            "mod submit_compiled_scene_frame;",
        ],
    );

    assert_contains_all(
        "render orchestration",
        render,
        &[
            "bind_compiled_scene_graph_resources(",
            "self.execute_compiled_scene_graph_stages(CompiledSceneGraphStageContext",
            "self.submit_compiled_scene_frame(CompiledSceneFrameSubmissionContext",
            "use super::pipeline_resource_usage::pipeline_writes_resource;",
            "use super::sprite_stage_selection::active_sprite_graph_stages;",
        ],
    );
    assert_not_contains(
        "render orchestration",
        render,
        &[
            "queue.submit([encoder.finish()])",
            "release_transient_backings_into_pool",
            "fn attach_scene_velocity_readback_stats",
            "fn active_late_graph_stages",
            "execute_graph_stage(",
            "fn active_sprite_graph_stages(",
            "fn pipeline_has_active_sprite_stage(",
            "fn pipeline_writes_resource(",
        ],
    );

    assert_contains_all(
        "compiled-scene sprite-stage selection owner",
        sprite_stage_selection,
        &[
            "pub(super) fn active_sprite_graph_stages(",
            "fn pipeline_has_active_sprite_stage(",
            "fn compiled_scene_sprite_stage_list_owns_core2d_product_stages(",
            "fn active_sprite_graph_stages_follow_unculled_sprite_passes(",
        ],
    );
    assert_not_contains(
        "compiled-scene sprite-stage selection owner",
        sprite_stage_selection,
        &["fn pipeline_writes_resource("],
    );
    assert_contains_all(
        "compiled-scene pipeline resource usage owner",
        pipeline_resource_usage,
        &[
            "pub(super) fn pipeline_writes_resource(",
            "RenderGraphResourceAccessKind::Write",
        ],
    );

    assert_contains_all(
        "compiled-scene resource binding owner",
        bind,
        &[
            "pub(super) struct CompiledSceneGraphResourceBindingFlags",
            "pub(super) fn bind_compiled_scene_graph_resources",
            "bind_frame_graph_resources(",
            "bind_history_graph_resources(",
            "materialize_transient_resources_with_pool",
            "bind_execution_owned_graph_resources(",
            "bind_plugin_graph_resources(",
        ],
    );
    assert_contains_all(
        "compiled-scene stage execution owner",
        execute,
        &[
            "pub(super) struct CompiledSceneGraphStageContext",
            "pub(super) fn execute_compiled_scene_graph_stages",
            "execute_graph_stage(",
            "self.render_scene_passes(",
            "self.copy_history_textures(",
            "fn active_late_graph_stages",
        ],
    );
    assert_contains_all(
        "compiled-scene submission owner",
        submit,
        &[
            "pub(super) struct CompiledSceneFrameSubmissionContext",
            "pub(super) fn submit_compiled_scene_frame",
            "queue.submit([encoder.finish()])",
            "attach_hzb_occlusion_readback_stats",
            "release_transient_backings_into_pool",
            "self.transient_resource_pool.end_frame()",
        ],
    );

    assert!(
        render.lines().count() < 500,
        "render.rs should remain below the F16 orchestration budget"
    );
    assert!(
        bind.lines().count() < 160,
        "bind_compiled_scene_graph_resources.rs should remain a focused owner"
    );
    assert!(
        execute.lines().count() < 420,
        "execute_compiled_scene_graph_stages.rs should remain a focused owner"
    );
    assert!(
        submit.lines().count() < 650,
        "submit_compiled_scene_frame.rs should remain a focused owner"
    );
    assert!(
        sprite_stage_selection.lines().count() < 180,
        "sprite_stage_selection.rs should remain a focused owner"
    );
    assert!(
        pipeline_resource_usage.lines().count() < 80,
        "pipeline_resource_usage.rs should remain a focused owner"
    );

    let docs = [
        review_findings,
        render_index,
        runtime_07,
        convention,
        framework_doc,
    ]
    .join("\n");
    assert_contains_all(
        "F16 status mirrors",
        &docs,
        &[
            "F16 render_compiled_scene structure split status guard",
            "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
            "review_f16_compiled_scene_render_path_uses_split_owners",
            "bind_compiled_scene_graph_resources.rs",
            "execute_compiled_scene_graph_stages.rs",
            "submit_compiled_scene_frame.rs",
            "sprite_stage_selection.rs",
            "pipeline_resource_usage.rs",
            "compiled_scene_render_split_coremin_and_focused_tests_passed",
        ],
    );
}
