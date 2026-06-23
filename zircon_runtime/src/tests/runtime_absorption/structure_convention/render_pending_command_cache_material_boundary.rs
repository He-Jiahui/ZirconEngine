use super::{assert_contains_all, runtime_src_path};

#[test]
fn runtime_15_pending_command_cache_material_bound_phases_stay_out_of_pre_mesh_rebuild() {
    let non_material_rebuild = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
    );
    let prepass_record =
        read_runtime_src("graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs");
    let shadow_renderer =
        read_runtime_src("graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs");
    let velocity_executor = read_runtime_src(
        "graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs",
    );
    let gpu_context = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
    );

    assert_contains_all(
        "pre-MeshDraw non-material rebuild stays limited to opaque shadow depth",
        &non_material_rebuild,
        &[
            "pub(super) fn can_rebuild_non_material_command_phase",
            "phase == RenderPhase::Shadow",
            "MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth",
            "MeshDrawQueuePhase::AlphaMask | MeshDrawQueuePhase::Transparent => return None",
            "fn depth_and_material_phases_are_not_pre_mesh_draw_rebuildable",
        ],
    );
    assert_contains_all(
        "normal prepass still requires standard material binding during replay",
        &prepass_record,
        &[
            "bind_gpu_scene_if_needed(pass, command",
            "bind_standard_material_if_needed(pass, command)",
            "bind_geometry_if_needed(pass, command)",
        ],
    );
    assert_contains_all(
        "alpha-mask shadow remains material-bound while opaque shadow depth does not",
        &shadow_renderer,
        &[
            "MeshPassPipelineKind::ShadowDepthAlphaMask",
            "bind_standard_material_if_needed(pass, command)",
            "MeshPassPipelineKind::ShadowDepth",
            "pass.set_pipeline(&self.pipeline)",
        ],
    );
    assert_contains_all(
        "object velocity pass remains material-bound",
        &velocity_executor,
        &[
            "ensure_velocity_pipeline_for_variant",
            "bind_standard_material_if_needed(pass, command)",
        ],
    );
    assert_contains_all(
        "TAA reactive mask pass remains material-bound",
        &gpu_context,
        &[
            "ensure_taa_reactive_mask_pipeline_for_variant",
            "bind_standard_material_if_needed(pass, command)",
        ],
    );
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}
