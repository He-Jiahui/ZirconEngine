use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, PostProcessPassGraph,
    RenderPostProcessEffectStackResourceStatus, RenderPostProcessEffectStackSettings,
};

pub(super) fn effect_stack_resource_status(
    post_process_graph: &PostProcessPassGraph,
    executed_executor_ids: &[String],
    motion_vector_camera_status: MotionVectorCameraStatus,
) -> RenderPostProcessEffectStackResourceStatus {
    let mut status = RenderPostProcessEffectStackResourceStatus {
        motion_vector_camera_status,
        ..Default::default()
    };
    for node in &post_process_graph.nodes {
        for resource in &node.required_inputs {
            match resource.as_str() {
                PostProcessGraphResourceNames::GBUFFER_NORMAL => {
                    status.ssr_normal_available = true;
                }
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION => {
                    status.ssr_temporal_history_available = true;
                }
                PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX => {
                    status.motion_vector_available = true;
                }
                _ => {}
            }
        }
    }
    for executor_id in executed_executor_ids {
        match executor_id.as_str() {
            "temporal.velocity-camera" => status.motion_vector_camera_available = true,
            "temporal.velocity-object" => status.motion_vector_object_available = true,
            "post.motion-vector-tile-max" => status.motion_vector_tile_max_available = true,
            "post.motion-vector-tile-max-coarse" => {
                status.motion_vector_tile_max_coarse_available = true;
            }
            "post.motion-vector-neighbor-max" => {
                status.motion_vector_neighbor_max_available = true;
            }
            _ => {}
        }
    }
    status.motion_vector_prepass_available = motion_vector_camera_status
        == MotionVectorCameraStatus::Ready
        && status.motion_vector_camera_available
        && status.motion_vector_object_available
        && status.motion_vector_tile_max_available
        && status.motion_vector_tile_max_coarse_available
        && status.motion_vector_neighbor_max_available;
    status
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
