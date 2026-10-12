use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner() {
    let root = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
    );
    let mesh_command_lists = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs",
    );
    let mesh_recording = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs",
    );

    let plan_01 = read_repo(
        "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let graph_execution_doc = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "GPU context root mounts mesh command-list owner and keeps WGPU pass context",
        &root,
        &[
            "mod mesh_command_lists;",
            "mod mesh_recording;",
            "use mesh_command_lists::RenderPassMeshCommandLists;",
            "pub struct RenderPassGpuExecutionContext",
        ],
    );

    for moved_owner in [
        "struct RenderPassMeshCommandLists",
        "fn stream_for_stage(",
        "fn depth_prepass_stream(",
        "fn shadow_stream(",
        "fn opaque_stream(",
        "fn alpha_mask_stream(",
        "fn transparent_stream(",
        "fn velocity_stream(",
        "fn taa_reactive_mask_stream(",
        "fn occlusion_cull_candidate_arg_count(",
        "fn occlusion_cull_candidate_instance_count(",
        "fn hzb_occlusion_indirect_executions(",
        "enum MeshStageCommandSource",
        "fn record_depth_prepass_to_resources(",
        "fn record_shadow_atlas_to_resources(",
        "fn record_mesh_stage_to_resources(",
        "fn record_mesh_stage_selection_to_resources(",
        "fn transmission_step_has_commands(",
        "fn record_taa_reactive_mask_mesh_to_resource(",
    ] {
        assert!(
            !root.contains(moved_owner),
            "gpu.rs should delegate mesh command-list owner `{moved_owner}`"
        );
    }

    assert_contains_all(
        "mesh recording child owns depth, shadow, material-stage, and reactive-mask recording",
        &mesh_recording,
        &[
            "enum MeshStageCommandSource",
            "pub(in crate::graphics::scene::scene_renderer) fn record_depth_prepass_to_resources(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_shadow_atlas_to_resources(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_mesh_stage_to_resources(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_advanced_pbr_opaque_to_resources(",
            "pub(in crate::graphics::scene::scene_renderer) fn transmission_step_has_commands(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_transmission_step_to_resources(",
            "fn record_mesh_stage_selection_to_resources(",
            "pub(in crate::graphics::scene::scene_renderer) fn record_taa_reactive_mask_mesh_to_resource(",
        ],
    );

    assert_contains_all(
        "mesh command-list child owns streams, indirect candidates, and HZB counters",
        &mesh_command_lists,
        &[
            "pub(in crate::graphics::scene::scene_renderer) struct RenderPassMeshCommandLists",
            "pub replay_stats: &'a MeshDrawReplayStatsAccumulator",
            "pub gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>",
            "pub advanced_pbr_opaque_commands: &'a [MeshDrawCommand]",
            "pub transmission_commands: &'a [MeshDrawCommand]",
            "pub transmission_step_count: usize",
            "pub advanced_pbr_opaque_indirect: Option<&'a MeshIndirectDrawExecution>",
            "fn stream_for_stage(",
            "fn depth_prepass_stream(",
            "fn shadow_stream(",
            "fn opaque_stream(",
            "fn alpha_mask_stream(",
            "fn transparent_stream(",
            "fn advanced_pbr_opaque_stream(",
            "fn transmission_step_stream(",
            "fn velocity_stream(",
            "fn taa_reactive_mask_stream(",
            "fn occlusion_cull_candidate_arg_count(",
            "fn occlusion_cull_candidate_instance_count(",
            "fn hzb_occlusion_indirect_executions(",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
            root.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs",
            mesh_command_lists.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs",
            mesh_recording.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the GPU context mesh command-list split, got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("graph execution docs", graph_execution_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 01 GPU context mesh command lists owner split",
                "render_plan01_gpu_context_mesh_command_lists_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
                "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs",
                "runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner",
            ],
        );
    }

    assert!(
        graph_execution_doc.contains(
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs"
        ),
        "graph execution module docs should list the mesh recording child owner"
    );
}
