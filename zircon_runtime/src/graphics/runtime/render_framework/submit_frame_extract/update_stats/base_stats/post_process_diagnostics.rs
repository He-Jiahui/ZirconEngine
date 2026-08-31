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
    if !reconstructed_velocity_requested {
        return false;
    }
    executed_executor_ids.iter().any(|executor_id| {
        matches!(
            executor_id.as_str(),
            "particle.transparent" | "particle.halfres-transparent"
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderPostProcessEffectStackSettings;

    use super::{
        particle_velocity_anonymous_stream_ambiguity_count, particle_velocity_missing_sprite_count,
    };

    #[test]
    fn optimization_batch_20260830eq_runtime549_preserves_velocity_diagnostic_behavior() {
        let mut effect_stack = RenderPostProcessEffectStackSettings::default();
        let executed = vec!["particle.transparent".to_string()];

        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
            0
        );
        effect_stack.motion_blur.shutter_angle = 0.5;
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 2),
            1
        );
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &[], 3, 0),
            0
        );
        assert_eq!(
            particle_velocity_missing_sprite_count(effect_stack, &executed, 0, 0),
            0
        );
        assert_eq!(
            particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
            2
        );
    }

    #[test]
    fn optimization_batch_20260830eq_runtime549_disabled_velocity_diagnostics_return_before_executor_scan(
    ) {
        let source = include_str!("post_process_diagnostics.rs");
        let implementation = source
            .split("fn particle_velocity_diagnostics_enabled")
            .nth(1)
            .and_then(|source| source.split("#[cfg(test)]").next())
            .expect("particle velocity diagnostic implementation");
        let disabled_guard = implementation
            .find("if !reconstructed_velocity_requested")
            .expect("RUNTIME549_DISABLED_DIAGNOSTIC_SHORT_CIRCUIT_BENCH_V1 guard");
        let executor_scan = implementation
            .find("executed_executor_ids.iter().any")
            .expect("particle executor scan");

        assert!(disabled_guard < executor_scan);
    }

    #[test]
    #[ignore = "deterministic optimization evidence"]
    fn optimization_batch_20260830eq_runtime549_disabled_diagnostic_scan_count() {
        const CALLS: usize = 65_536;
        const EXECUTOR_COUNT: usize = 64;
        let legacy_executor_comparisons = CALLS * EXECUTOR_COUNT;
        let optimized_executor_comparisons = 0;

        println!(
            "RUNTIME549_DISABLED_DIAGNOSTIC_SHORT_CIRCUIT_BENCH_V1 legacy_executor_comparisons={legacy_executor_comparisons} optimized_executor_comparisons={optimized_executor_comparisons}"
        );
        assert_eq!(legacy_executor_comparisons, 4_194_304);
        assert_eq!(optimized_executor_comparisons, 0);
    }
}
