use super::*;

#[test]
fn render_framework_tracks_viewports_and_accepts_frame_extract_submission() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, RenderQualityProfile::new("editor"))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(viewport, RenderViewportHandle::new(1));
    assert_eq!(stats.active_viewports, 1);
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
    assert_eq!(
        stats.last_frame_history_status.current,
        Some(FrameHistoryHandle::new(1))
    );
    assert_eq!(stats.last_frame_history_status.previous, None);
    assert!(!stats.last_frame_history_status.previous_available);
    assert_eq!(
        stats.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::NoPreviousFrame)
    );
    assert_eq!(stats.capabilities.backend_name, "wgpu");
    assert!(!stats.capabilities.supports_surface);
    assert!(stats.capabilities.supports_offscreen);
    assert!(!stats.capabilities.acceleration_structures_supported);
}

#[test]
fn render_framework_stats_report_scene_camera_ordering_metadata() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = test_extract();
    extract.view.scene_camera_order_report = Some(RenderCameraOrderReport {
        cameras: vec![
            SortedRenderCamera {
                entity: 10,
                camera: CameraRenderDescriptor::from_camera_payload(
                    Some(10),
                    ViewportCameraSnapshot::default(),
                ),
                render_type: CameraRenderType::Base,
                order: 0,
                target: RenderCameraTargetOrderKey::PrimarySurface,
                hdr: false,
                sorted_camera_index_for_target: 0,
            },
            SortedRenderCamera {
                entity: 11,
                camera: CameraRenderDescriptor::from_camera_payload(
                    Some(11),
                    ViewportCameraSnapshot::default(),
                ),
                render_type: CameraRenderType::Base,
                order: 0,
                target: RenderCameraTargetOrderKey::PrimarySurface,
                hdr: false,
                sorted_camera_index_for_target: 1,
            },
        ],
        ambiguities: vec![RenderCameraOrderAmbiguity {
            order: 0,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }],
    });

    server.submit_frame_extract(viewport, extract).unwrap();

    let stats = server.query_stats().unwrap();
    assert_eq!(stats.last_scene_camera_scheduled_count, 2);
    assert_eq!(stats.last_scene_camera_order_ambiguity_count, 1);
}

#[test]
fn render_framework_uses_default_forward_plus_pipeline_when_viewport_has_no_explicit_pipeline() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(1)),
        "submit should fall back to the default Forward+ pipeline asset"
    );
}

#[test]
fn render_framework_stats_report_executed_render_graph_passes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let expected_pipeline = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_async_compute(false),
        )
        .unwrap();
    let expected_executed_passes = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled && pass.executor_id.is_some())
        .map(|pass| pass.name.clone())
        .collect::<Vec<_>>();
    let expected_executor_ids = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled)
        .filter_map(|pass| pass.executor_id.clone())
        .collect::<Vec<_>>();
    let expected_resource_access_count = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled && pass.executor_id.is_some())
        .map(|pass| pass.resources.len())
        .sum::<usize>();
    let expected_dependency_count = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| !pass.culled && pass.executor_id.is_some())
        .map(|pass| pass.dependencies.len())
        .sum::<usize>();
    let expected_graph_stats = expected_pipeline.graph().stats();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_graph_pass_count,
        expected_pipeline.graph().passes().len()
    );
    assert_eq!(
        stats.last_graph_culled_pass_count,
        expected_pipeline
            .graph()
            .passes()
            .iter()
            .filter(|pass| pass.culled)
            .count()
    );
    assert_eq!(
        stats.last_graph_executed_pass_count,
        expected_executed_passes.len()
    );
    assert_eq!(stats.last_graph_executed_passes, expected_executed_passes);
    assert_eq!(
        stats.last_graph_executed_executor_ids,
        expected_executor_ids
    );
    assert_eq!(
        stats.last_graph_executed_resource_access_count,
        expected_resource_access_count
    );
    assert_eq!(
        stats.last_graph_executed_dependency_count,
        expected_dependency_count
    );
    assert_eq!(
        stats.last_graph_resource_lifetime_count,
        expected_graph_stats.resource_lifetime_count
    );
    assert_eq!(
        stats.last_graph_sparse_texture_lifetime_count,
        expected_graph_stats.sparse_texture_lifetime_count
    );
    assert_eq!(
        stats.last_graph_planned_resource_access_count,
        expected_graph_stats.total_resource_access_count
    );
    assert_eq!(
        stats.last_graph_planned_dependency_count,
        expected_graph_stats.total_dependency_count
    );
    let expected_allocation_plan = expected_pipeline.graph().transient_allocation_plan();
    assert_eq!(
        stats.last_graph_transient_texture_slot_count,
        expected_allocation_plan.texture_slot_count
    );
    assert_eq!(
        stats.last_graph_sparse_texture_slot_count,
        expected_allocation_plan.sparse_texture_slot_count
    );
    assert_eq!(
        stats.last_graph_transient_buffer_slot_count,
        expected_allocation_plan.buffer_slot_count
    );
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
    assert_eq!(
        stats.last_graph_executed_passes.first().map(String::as_str),
        Some("preview-sky")
    );
    assert_eq!(
        stats.last_graph_executed_passes.get(1).map(String::as_str),
        Some("depth-prepass")
    );
    assert!(
        stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "overlay-gizmo")
    );
}

#[test]
fn render_framework_stats_report_executed_product_postprocess_nodes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = test_extract();
    extract.post_process.bloom = RenderBloomSettings {
        threshold: 0.6,
        intensity: 1.0,
        radius: 0.8,
    };
    extract.post_process.color_grading = RenderColorGradingSettings {
        exposure: 1.05,
        contrast: 1.1,
        saturation: 0.9,
        gamma: 1.0,
        tint: Vec3::new(1.08, 0.95, 0.9),
    };

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("post-process-product")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(stats.last_post_process_graph_node_count, 4);
    assert_eq!(stats.last_post_process_graph_skipped_node_count, 1);
    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_eq!(
        stats.last_post_process_graph_executed_nodes,
        vec![
            "bloom".to_string(),
            "color-lut-bake".to_string(),
            "output-transfer".to_string(),
            "fxaa".to_string(),
        ]
    );
    assert!(
        stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == "overlay-gizmo")
    );
}

#[test]
fn render_framework_stats_report_neutral_color_lut_readback_identity() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("post-process-neutral-lut")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_anti_alias(false),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|node| node == "color-lut-bake")
    );
    let report = stats.last_color_lut_readback_report;
    assert!(report.available, "color LUT readback was not available");
    assert_eq!(
        report.size,
        [
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT,
            COLOR_LUT_SIZE_DEFAULT
        ]
    );
    assert_eq!(report.sample_count, COLOR_LUT_SIZE_DEFAULT.pow(3) as usize);
    assert!(
        report.identity_within_epsilon(),
        "neutral color LUT readback exceeded identity tolerance: {report:?}"
    );
}

#[test]
fn render_framework_stats_report_effect_stack_product_node_when_authored() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = test_extract();
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: None,
            intensity: 0.5,
            ..Default::default()
        },
        depth_of_field: RenderDepthOfFieldSettings {
            aperture: 0.75,
            max_blur_radius: 2.0,
            ..Default::default()
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.4,
            max_steps: 16,
            ..Default::default()
        },
        vignette: RenderVignetteSettings {
            intensity: 0.4,
            ..Default::default()
        },
        grain: RenderFilmGrainSettings {
            intensity: 0.1,
            ..Default::default()
        },
        dither: RenderDitherSettings {
            intensity: 0.05,
            ..Default::default()
        },
        chromatic_aberration: RenderChromaticAberrationSettings {
            intensity: 0.2,
            ..Default::default()
        },
        fog: RenderFogSettings {
            density: 0.08,
            ..Default::default()
        },
        ..Default::default()
    };

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("post-process-effect-stack")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(stats.last_post_process_graph_node_count, 2);
    assert_eq!(
        stats.last_post_process_graph_executed_nodes,
        vec!["uber".to_string(), "output-transfer".to_string()]
    );
    assert!(stats.last_post_process_effect_stack_report.enabled);
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec![
            "lut".to_string(),
            "depth-of-field".to_string(),
            "screen-space-reflection".to_string(),
            "vignette".to_string(),
            "film-grain".to_string(),
            "dither".to_string(),
            "chromatic-aberration".to_string(),
            "fog".to_string(),
        ]
    );
    assert_eq!(
        stats
            .last_post_process_effect_stack_report
            .approximated_families,
        vec![
            "depth-of-field".to_string(),
            "screen-space-reflection".to_string(),
        ]
    );
    assert_eq!(
        stats
            .last_post_process_effect_stack_report
            .missing_resources,
        vec!["effect-stack.lut.texture".to_string()]
    );
    assert_eq!(stats.last_post_process_lut_request_count, 1);
    assert_eq!(stats.last_post_process_lut_ready_count, 0);
    assert_eq!(stats.last_post_process_lut_fallback_count, 1);
    assert_eq!(stats.last_post_process_lut_2d_strip_ready_count, 0);
    assert_eq!(stats.last_post_process_lut_3d_request_count, 0);
    assert_eq!(stats.last_post_process_lut_unsupported_shape_count, 0);
}

#[test]
fn render_framework_stats_report_volume_effect_stack_product_node_when_authored() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut extract = test_extract();
    let profile = RenderPostProcessVolumeProfile::default().with_effect_stack(
        RenderPostProcessEffectStackSettings {
            vignette: RenderVignetteSettings {
                intensity: 0.6,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    extract.post_process.volumes = vec![PostProcessVolumeExtract::global(
        0.0,
        0.5,
        extract.view.selected_camera_layers().clone(),
        VolumeComponentOverride::from_profile(&profile),
    )];

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("post-process-volume-effect-stack")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(stats.last_post_process_graph_node_count, 2);
    assert_eq!(
        stats.last_post_process_graph_executed_nodes,
        vec!["uber".to_string(), "output-transfer".to_string()]
    );
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["vignette".to_string()]
    );
    assert!(
        stats
            .last_post_process_effect_stack_report
            .missing_resources
            .is_empty()
    );
}
