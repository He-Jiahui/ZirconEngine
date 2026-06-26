use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, PostProcessPassGraph,
    RenderPostProcessEffectStackResourceStatus, RenderPostProcessEffectStackSettings,
};

pub(super) fn effect_stack_resource_status(
    post_process_graph: &PostProcessPassGraph,
    executed_executor_ids: &[String],
    motion_vector_camera_status: MotionVectorCameraStatus,
) -> RenderPostProcessEffectStackResourceStatus {
    let ssr_normal_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
    );
    let ssr_temporal_history_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
    );
    let motion_vector_available = effect_stack_uses_resource(
        post_process_graph,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
    );
    let motion_vector_camera_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "temporal.velocity-camera");
    let motion_vector_object_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "temporal.velocity-object");
    let motion_vector_tile_max_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-tile-max");
    let motion_vector_tile_max_coarse_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-tile-max-coarse");
    let motion_vector_neighbor_max_available = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "post.motion-vector-neighbor-max");
    RenderPostProcessEffectStackResourceStatus {
        ssr_normal_available,
        ssr_temporal_history_available,
        motion_vector_available,
        motion_vector_camera_available,
        motion_vector_object_available,
        motion_vector_tile_max_available,
        motion_vector_tile_max_coarse_available,
        motion_vector_neighbor_max_available,
        motion_vector_camera_status,
        motion_vector_prepass_available: motion_vector_camera_status
            == MotionVectorCameraStatus::Ready
            && motion_vector_camera_available
            && motion_vector_object_available
            && motion_vector_tile_max_available
            && motion_vector_tile_max_coarse_available
            && motion_vector_neighbor_max_available,
    }
}

fn effect_stack_uses_resource(
    post_process_graph: &PostProcessPassGraph,
    resource_name: &str,
) -> bool {
    post_process_graph.nodes.iter().any(|node| {
        node.required_inputs
            .iter()
            .any(|resource| resource == resource_name)
    })
}

pub(super) fn particle_velocity_missing_sprite_count(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
    particle_previous_state_sprite_count: usize,
) -> usize {
    if particle_velocity_diagnostics_enabled(
        effect_stack,
        executed_executor_ids,
        particle_sprite_count,
    ) {
        particle_sprite_count.saturating_sub(particle_previous_state_sprite_count)
    } else {
        0
    }
}

pub(super) fn particle_velocity_anonymous_stream_ambiguity_count(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
    particle_anonymous_stream_ambiguity_sprite_count: usize,
) -> usize {
    if particle_velocity_diagnostics_enabled(
        effect_stack,
        executed_executor_ids,
        particle_sprite_count,
    ) {
        particle_anonymous_stream_ambiguity_sprite_count
    } else {
        0
    }
}

fn particle_velocity_diagnostics_enabled(
    effect_stack: RenderPostProcessEffectStackSettings,
    executed_executor_ids: &[String],
    particle_sprite_count: usize,
) -> bool {
    if particle_sprite_count == 0 {
        return false;
    }
    let reconstructed_velocity_requested =
        effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled();
    let particle_transparent_executed = executed_executor_ids
        .iter()
        .any(|executor_id| executor_id == "particle.transparent");
    reconstructed_velocity_requested && particle_transparent_executed
}
