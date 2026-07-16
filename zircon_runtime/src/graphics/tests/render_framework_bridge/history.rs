use super::*;

#[test]
fn render_framework_records_temporal_history_after_compatible_history_exists() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("history-product")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(true)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let first_stats = server.query_stats().unwrap();
    assert_eq!(
        first_stats.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::NoPreviousFrame)
    );
    assert!(!first_stats.last_frame_history_status.previous_available);
    assert!(!first_stats
        .last_post_process_graph_executed_nodes
        .contains(&"taa-resolve".to_string()));

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let second_stats = server.query_stats().unwrap();
    assert!(second_stats.last_frame_history_status.previous_available);
    assert_eq!(
        second_stats.last_frame_history_status.invalidation_reason,
        None
    );
    assert!(!second_stats
        .last_post_process_graph_executed_nodes
        .contains(&"taa-resolve".to_string()));
}

#[test]
fn render_framework_tracks_text_payloads_submitted_with_shared_ui_extracts() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract_with_ui(
            viewport,
            test_extract(),
            Some(test_ui_extract("Editor HUD")),
        )
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(stats.last_ui_command_count, 1);
    assert_eq!(stats.last_ui_quad_count, 1);
    assert_eq!(stats.last_ui_text_payload_count, 1);
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&"ui.screen-space".to_string()));
    assert_eq!(stats.last_ui_graph_executed_pass_count, 1);
    assert_eq!(stats.last_ui_target_size, Some(UVec2::new(320, 240)));
    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-overlay-ui")
    );
}

#[test]
fn render_framework_reuses_frame_history_handle_for_compatible_submissions() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let first = server.query_stats().unwrap().last_frame_history;

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();
    let second = stats.last_frame_history;

    assert_eq!(first, second);
    assert_eq!(second, Some(FrameHistoryHandle::new(1)));
    assert_eq!(stats.last_frame_history_status.current, second);
    assert_eq!(stats.last_frame_history_status.previous, first);
    assert!(stats.last_frame_history_status.previous_available);
    assert_eq!(stats.last_frame_history_status.invalidation_reason, None);
}

#[test]
fn render_framework_reports_frame_history_invalidation_when_camera_moves() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("history-validity")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(true)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let compatible = server.query_stats().unwrap();
    assert_eq!(
        compatible.last_frame_history,
        Some(FrameHistoryHandle::new(1))
    );
    assert!(compatible.last_frame_history_status.previous_available);
    assert!(!compatible
        .last_post_process_graph_executed_nodes
        .contains(&"taa-resolve".to_string()));

    let mut moved_camera = test_extract();
    moved_camera.view.camera.transform = Transform::from_translation(Vec3::new(0.25, 0.0, 0.0));
    server.submit_frame_extract(viewport, moved_camera).unwrap();
    let invalidated = server.query_stats().unwrap();

    assert_eq!(
        invalidated.last_frame_history,
        Some(FrameHistoryHandle::new(1))
    );
    assert_eq!(
        invalidated.last_frame_history_status.current,
        Some(FrameHistoryHandle::new(1))
    );
    assert_eq!(
        invalidated.last_frame_history_status.previous,
        Some(FrameHistoryHandle::new(1))
    );
    assert!(!invalidated.last_frame_history_status.previous_available);
    assert_eq!(
        invalidated.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::FrameInputsChanged)
    );
    assert!(!invalidated
        .last_post_process_graph_executed_nodes
        .contains(&"taa-resolve".to_string()));
}

#[test]
fn render_framework_invalidates_history_when_dynamic_render_size_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("history-dynamic-resolution")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(true)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let compatible = server.query_stats().unwrap();
    assert_eq!(compatible.last_frame_target_size, Some(viewport_size));
    assert_eq!(compatible.last_frame_render_size, Some(viewport_size));
    assert!(compatible.last_frame_history_status.previous_available);

    let mut scaled = test_extract();
    scaled.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    server.submit_frame_extract(viewport, scaled).unwrap();
    let invalidated = server.query_stats().unwrap();

    assert_eq!(invalidated.last_frame_target_size, Some(viewport_size));
    assert_eq!(
        invalidated.last_frame_render_size,
        Some(UVec2::new(160, 120))
    );
    assert_eq!(
        invalidated.last_frame_history_status.target_size,
        viewport_size
    );
    assert_eq!(
        invalidated.last_frame_history_status.render_size,
        UVec2::new(160, 120)
    );
    assert_eq!(
        invalidated.last_frame_history,
        Some(FrameHistoryHandle::new(2))
    );
    assert_eq!(
        invalidated.last_frame_history_status.previous,
        Some(FrameHistoryHandle::new(1))
    );
    assert!(!invalidated.last_frame_history_status.previous_available);
    assert_eq!(
        invalidated.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::RenderSizeChanged)
    );
    assert!(!invalidated
        .last_post_process_graph_executed_nodes
        .contains(&"taa-resolve".to_string()));
}
