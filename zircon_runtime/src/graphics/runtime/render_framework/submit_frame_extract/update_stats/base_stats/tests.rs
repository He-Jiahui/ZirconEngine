use super::{
    effect_stack_resource_status, graph_execution_coverage_report_from_names,
    particle_velocity_anonymous_stream_ambiguity_count, particle_velocity_missing_sprite_count,
    update_hzb_occlusion_stats, update_visibility_static_index_stats, update_visibility_stats,
};
use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessPassGraph, PostProcessPassNode, RenderPostProcessEffectStackSettings, RenderStats,
};
use crate::graphics::visibility::{
    FrameVisibility, HzbOcclusionCullReadbackStats, HzbOcclusionCullReport, ViewCullingStats,
    ViewVisibilityContext, VisibilityStaticIndexReport,
};

fn effect_stack_graph(required_inputs: Vec<String>) -> PostProcessPassGraph {
    PostProcessPassGraph::from_ordered_nodes(
        vec![
            PostProcessPassNode::new("uber", PostProcessEffectKind::Uber)
                .with_required_inputs(required_inputs),
        ],
        Vec::new(),
        None,
    )
}

#[test]
fn graph_execution_coverage_report_counts_missing_unexpected_and_duplicate_passes() {
    let executed_passes = vec![
        "depth-prepass".to_string(),
        "depth-prepass".to_string(),
        "uber".to_string(),
        "unexpected-pass".to_string(),
    ];

    let report = graph_execution_coverage_report_from_names(
        ["depth-prepass", "opaque", "uber"],
        &executed_passes,
    );

    assert_eq!(report.planned_live_pass_count, 3);
    assert_eq!(report.executed_pass_count, 4);
    assert_eq!(report.matched_planned_pass_count, 2);
    assert_eq!(report.missing_planned_pass_count, 1);
    assert_eq!(report.unexpected_executed_pass_count, 1);
    assert_eq!(report.duplicate_executed_pass_count, 1);
}

#[test]
fn update_visibility_stats_sums_per_view_culling_counts() {
    let frame_visibility = FrameVisibility {
        views: vec![
            ViewVisibilityContext {
                stats: ViewCullingStats {
                    input_count: 4,
                    layer_filtered_count: 1,
                    frustum_culled_count: 1,
                    occlusion_culled_count: 0,
                    visible_count: 2,
                },
                ..ViewVisibilityContext::default()
            },
            ViewVisibilityContext {
                stats: ViewCullingStats {
                    input_count: 4,
                    layer_filtered_count: 0,
                    frustum_culled_count: 2,
                    occlusion_culled_count: 1,
                    visible_count: 1,
                },
                ..ViewVisibilityContext::default()
            },
        ],
        ..FrameVisibility::default()
    };
    let mut stats = RenderStats::default();

    update_visibility_stats(&mut stats, &frame_visibility);

    assert_eq!(stats.last_visibility_view_count, 2);
    assert_eq!(stats.last_visibility_input_count, 8);
    assert_eq!(stats.last_visibility_layer_filtered_count, 1);
    assert_eq!(stats.last_visibility_frustum_culled_count, 3);
    assert_eq!(stats.last_visibility_occlusion_culled_count, 1);
    assert_eq!(stats.last_visibility_visible_count, 3);
}

#[test]
fn update_visibility_static_index_stats_records_latest_report() {
    let mut stats = RenderStats::default();
    let report = VisibilityStaticIndexReport {
        frame_full_rebuild_count: 0,
        frame_incremental_update_count: 1,
        inserted_count: 2,
        updated_count: 3,
        removed_count: 4,
        indexed_entity_count: 10,
        occupied_cell_count: 7,
        main_view_prefilter_used: true,
        main_view_static_input_count: 12,
        main_view_static_candidate_count: 5,
        ..VisibilityStaticIndexReport::default()
    };

    update_visibility_static_index_stats(&mut stats, &report);

    assert_eq!(stats.last_visibility_static_index_full_rebuild_count, 0);
    assert_eq!(
        stats.last_visibility_static_index_incremental_update_count,
        1
    );
    assert_eq!(stats.last_visibility_static_index_inserted_count, 2);
    assert_eq!(stats.last_visibility_static_index_updated_count, 3);
    assert_eq!(stats.last_visibility_static_index_removed_count, 4);
    assert_eq!(stats.last_visibility_static_index_indexed_entity_count, 10);
    assert_eq!(stats.last_visibility_static_index_occupied_cell_count, 7);
    assert!(stats.last_visibility_static_index_main_view_prefilter_used);
    assert_eq!(
        stats.last_visibility_static_index_main_view_static_input_count,
        12
    );
    assert_eq!(
        stats.last_visibility_static_index_main_view_static_candidate_count,
        5
    );
}

#[test]
fn update_hzb_occlusion_stats_records_latest_cull_report() {
    let mut stats = RenderStats::default();
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 2, 1, true);

    update_hzb_occlusion_stats(&mut stats, Some(report));

    assert!(stats.last_hzb_occlusion_reported);
    assert_eq!(stats.last_hzb_occlusion_candidate_arg_count, 6);
    assert_eq!(stats.last_hzb_occlusion_candidate_instance_count, 42);
    assert_eq!(stats.last_hzb_occlusion_dispatch_group_count, 2);
    assert_eq!(stats.last_hzb_occlusion_dispatched_phase_count, 1);
    assert!(stats.last_hzb_occlusion_history_available);
    assert!(!stats.last_hzb_occlusion_readback_available);
}

#[test]
fn update_hzb_occlusion_stats_records_readback_and_overrides_visibility_occlusion_count() {
    let mut stats = RenderStats {
        last_visibility_occlusion_culled_count: 3,
        ..RenderStats::default()
    };
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 2, 1, true)
        .with_readback_stats(HzbOcclusionCullReadbackStats::new(6, 42, 2, 18))
        .with_indirect_args_readback(
            crate::graphics::visibility::HzbOcclusionIndirectArgsReadbackSummary::new(6, 4, 2, 24),
        );

    update_hzb_occlusion_stats(&mut stats, Some(report));

    assert!(stats.last_hzb_occlusion_readback_available);
    assert_eq!(stats.last_hzb_occlusion_tested_arg_count, 6);
    assert_eq!(stats.last_hzb_occlusion_tested_instance_count, 42);
    assert_eq!(stats.last_hzb_occlusion_culled_arg_count, 2);
    assert_eq!(stats.last_hzb_occlusion_culled_instance_count, 18);
    assert!(stats.last_hzb_occlusion_indirect_args_readback_available);
    assert_eq!(stats.last_hzb_occlusion_readback_arg_count, 6);
    assert_eq!(stats.last_hzb_occlusion_compacted_draw_count, 4);
    assert_eq!(stats.last_hzb_occlusion_zero_instance_arg_count, 2);
    assert_eq!(stats.last_hzb_occlusion_remaining_instance_count, 24);
    assert_eq!(stats.last_visibility_occlusion_culled_count, 18);
}

#[test]
fn update_hzb_occlusion_stats_resets_when_no_report() {
    let mut stats = RenderStats {
        last_hzb_occlusion_reported: true,
        last_hzb_occlusion_candidate_arg_count: 6,
        last_hzb_occlusion_candidate_instance_count: 42,
        last_hzb_occlusion_dispatch_group_count: 2,
        last_hzb_occlusion_dispatched_phase_count: 1,
        last_hzb_occlusion_history_available: true,
        last_hzb_occlusion_readback_available: true,
        last_hzb_occlusion_tested_arg_count: 6,
        last_hzb_occlusion_tested_instance_count: 42,
        last_hzb_occlusion_culled_arg_count: 2,
        last_hzb_occlusion_culled_instance_count: 18,
        last_hzb_occlusion_indirect_args_readback_available: true,
        last_hzb_occlusion_readback_arg_count: 6,
        last_hzb_occlusion_compacted_draw_count: 4,
        last_hzb_occlusion_zero_instance_arg_count: 2,
        last_hzb_occlusion_remaining_instance_count: 24,
        ..RenderStats::default()
    };

    update_hzb_occlusion_stats(&mut stats, None);

    assert!(!stats.last_hzb_occlusion_reported);
    assert_eq!(stats.last_hzb_occlusion_candidate_arg_count, 0);
    assert_eq!(stats.last_hzb_occlusion_candidate_instance_count, 0);
    assert_eq!(stats.last_hzb_occlusion_dispatch_group_count, 0);
    assert_eq!(stats.last_hzb_occlusion_dispatched_phase_count, 0);
    assert!(!stats.last_hzb_occlusion_history_available);
    assert!(!stats.last_hzb_occlusion_readback_available);
    assert_eq!(stats.last_hzb_occlusion_tested_arg_count, 0);
    assert_eq!(stats.last_hzb_occlusion_tested_instance_count, 0);
    assert_eq!(stats.last_hzb_occlusion_culled_arg_count, 0);
    assert_eq!(stats.last_hzb_occlusion_culled_instance_count, 0);
    assert!(!stats.last_hzb_occlusion_indirect_args_readback_available);
    assert_eq!(stats.last_hzb_occlusion_readback_arg_count, 0);
    assert_eq!(stats.last_hzb_occlusion_compacted_draw_count, 0);
    assert_eq!(stats.last_hzb_occlusion_zero_instance_arg_count, 0);
    assert_eq!(stats.last_hzb_occlusion_remaining_instance_count, 0);
}

#[test]
fn effect_stack_resource_status_detects_graph_bound_ssr_normal() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()
    ]);

    assert!(
        effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested)
            .ssr_normal_available
    );
}

#[test]
fn effect_stack_resource_status_detects_graph_bound_ssr_temporal_history_without_prepass() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string(),
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
    ]);

    let status = effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested);

    assert!(status.ssr_temporal_history_available);
    assert!(status.motion_vector_available);
    assert!(!status.motion_vector_prepass_available);
}

#[test]
fn effect_stack_resource_status_detects_graph_bound_motion_vector_neighbor_max_without_prepass() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
    ]);

    let status = effect_stack_resource_status(&graph, &[], MotionVectorCameraStatus::NotRequested);

    assert!(status.motion_vector_available);
    assert!(!status.motion_vector_camera_available);
    assert!(!status.motion_vector_object_available);
    assert!(!status.motion_vector_tile_max_available);
    assert!(!status.motion_vector_tile_max_coarse_available);
    assert!(!status.motion_vector_neighbor_max_available);
    assert!(!status.motion_vector_prepass_available);
}

#[test]
fn effect_stack_resource_status_detects_split_motion_blur_node_motion_vectors() {
    let graph = PostProcessPassGraph::from_ordered_nodes(
        vec![
            PostProcessPassNode::new("motion-blur", PostProcessEffectKind::MotionBlur)
                .with_required_inputs(vec![
                    PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
                ]),
            PostProcessPassNode::new("uber", PostProcessEffectKind::Uber),
        ],
        Vec::new(),
        None,
    );
    let executed = vec![
        "temporal.velocity-camera".to_string(),
        "temporal.velocity-object".to_string(),
        "post.motion-vector-tile-max".to_string(),
        "post.motion-vector-tile-max-coarse".to_string(),
        "post.motion-vector-neighbor-max".to_string(),
    ];

    let status = effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

    assert!(status.motion_vector_available);
    assert!(status.motion_vector_prepass_available);
}

#[test]
fn effect_stack_resource_status_detects_executed_motion_vector_prepass_chain() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
    ]);
    let executed = vec![
        "temporal.velocity-camera".to_string(),
        "temporal.velocity-object".to_string(),
        "post.motion-vector-tile-max".to_string(),
        "post.motion-vector-tile-max-coarse".to_string(),
        "post.motion-vector-neighbor-max".to_string(),
    ];

    let status = effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

    assert!(status.motion_vector_available);
    assert!(status.motion_vector_camera_available);
    assert!(status.motion_vector_object_available);
    assert!(status.motion_vector_tile_max_available);
    assert!(status.motion_vector_tile_max_coarse_available);
    assert!(status.motion_vector_neighbor_max_available);
    assert_eq!(
        status.motion_vector_camera_status,
        MotionVectorCameraStatus::Ready
    );
    assert!(status.motion_vector_prepass_available);
}

#[test]
fn effect_stack_resource_status_keeps_prepass_unavailable_without_object_vectors() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
    ]);
    let executed = vec![
        "temporal.velocity-camera".to_string(),
        "post.motion-vector-tile-max".to_string(),
        "post.motion-vector-tile-max-coarse".to_string(),
        "post.motion-vector-neighbor-max".to_string(),
    ];

    let status = effect_stack_resource_status(&graph, &executed, MotionVectorCameraStatus::Ready);

    assert!(status.motion_vector_available);
    assert!(status.motion_vector_camera_available);
    assert!(!status.motion_vector_object_available);
    assert!(status.motion_vector_tile_max_available);
    assert!(status.motion_vector_tile_max_coarse_available);
    assert!(status.motion_vector_neighbor_max_available);
    assert!(!status.motion_vector_prepass_available);
}

#[test]
fn effect_stack_resource_status_keeps_prepass_unavailable_when_camera_vectors_are_cut() {
    let graph = effect_stack_graph(vec![
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string(),
    ]);
    let executed = vec![
        "temporal.velocity-camera".to_string(),
        "temporal.velocity-object".to_string(),
        "post.motion-vector-tile-max".to_string(),
        "post.motion-vector-tile-max-coarse".to_string(),
        "post.motion-vector-neighbor-max".to_string(),
    ];

    let status = effect_stack_resource_status(
        &graph,
        &executed,
        MotionVectorCameraStatus::CameraCutOrInvalid,
    );

    assert!(status.motion_vector_camera_available);
    assert!(status.motion_vector_object_available);
    assert_eq!(
        status.motion_vector_camera_status,
        MotionVectorCameraStatus::CameraCutOrInvalid
    );
    assert!(!status.motion_vector_prepass_available);
}

#[test]
fn particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested() {
    let mut effect_stack = RenderPostProcessEffectStackSettings::default();
    let executed = vec!["particle.transparent".to_string()];

    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
        0
    );

    effect_stack.motion_blur.shutter_angle = 0.5;

    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
        3
    );
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &[], 3, 0),
        0
    );
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 0, 0),
        0
    );
}

#[test]
fn particle_velocity_gap_excludes_sprites_with_previous_state() {
    let mut effect_stack = RenderPostProcessEffectStackSettings::default();
    effect_stack.motion_blur.shutter_angle = 0.5;
    let executed = vec!["particle.transparent".to_string()];

    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 2),
        1
    );
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 8),
        0
    );
}

#[test]
fn particle_velocity_anonymous_stream_ambiguity_requires_velocity_diagnostics() {
    let mut effect_stack = RenderPostProcessEffectStackSettings::default();
    let executed = vec!["particle.transparent".to_string()];

    assert_eq!(
        particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
        0
    );

    effect_stack.motion_blur.shutter_angle = 0.5;

    assert_eq!(
        particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
        2
    );
    assert_eq!(
        particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &[], 2, 2),
        0
    );
    assert_eq!(
        particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 0, 2),
        0
    );
}
