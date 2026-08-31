use std::collections::BTreeSet;

use super::*;

#[test]
fn headless_wgpu_server_falls_back_async_compute_passes_to_graphics() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let extract = test_extract();
    let expected_pipeline = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_async_compute(false),
        )
        .unwrap();

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    let executed_passes = stats
        .last_graph_executed_passes
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_executed_fallback_pass_count = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| {
            executed_passes.contains(pass.name.as_str()) && pass.declared_queue != pass.queue
        })
        .count();
    let expected_executed_compute_workload_count = expected_pipeline
        .graph()
        .passes()
        .iter()
        .filter(|pass| {
            executed_passes.contains(pass.name.as_str()) && pass.compute_workload.is_some()
        })
        .count();

    assert!(!stats.capabilities.supports_async_compute);
    assert_eq!(stats.last_async_compute_pass_count, 0);
    assert!(expected_executed_fallback_pass_count > 0);
    assert_eq!(
        stats.last_graph_queue_fallback_pass_count,
        expected_executed_fallback_pass_count
    );
    assert!(expected_executed_compute_workload_count > 0);
    assert_eq!(
        stats.last_graph_compute_dispatch_count,
        expected_executed_compute_workload_count
    );
    assert!(
        stats.last_graph_compute_dispatch_group_count > 0,
        "clustered lighting and HZB should record concrete compute dispatch group evidence"
    );
    assert!(
        stats.last_graph_compute_storage_write_resource_count
            >= expected_executed_compute_workload_count,
        "each executed compute workload should record at least one storage write resource"
    );
    assert_eq!(
        stats.last_graph_compute_planned_workload_count,
        expected_executed_compute_workload_count
    );
    assert_eq!(
        stats.last_graph_compute_matched_workload_count,
        expected_executed_compute_workload_count
    );
    assert_eq!(stats.last_graph_compute_missing_dispatch_count, 0);
    assert_eq!(stats.last_graph_compute_workload_mismatch_count, 0);
    assert_eq!(stats.last_graph_compute_unexpected_dispatch_count, 0);
    assert!(
        stats
            .last_effective_features
            .contains(&"clustered_lighting".to_string()),
        "clustered lighting should stay enabled while queue execution falls back to graphics"
    );
}

#[test]
fn render_framework_rotates_frame_history_handle_when_pipeline_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let forward_history = server.query_stats().unwrap().last_frame_history;

    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let deferred_history = server.query_stats().unwrap().last_frame_history;

    assert_ne!(forward_history, deferred_history);
    assert_eq!(forward_history, Some(FrameHistoryHandle::new(1)));
    assert_eq!(deferred_history, Some(FrameHistoryHandle::new(2)));
}

#[test]
fn quality_profile_can_disable_ssao_clustered_and_history_features() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let before = server.query_stats().unwrap().last_frame_history;

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("forward-lite")
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_temporal_history(false),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_ne!(before, stats.last_frame_history);
    assert!(
        !stats
            .last_effective_features
            .contains(&"screen_space_ambient_occlusion".to_string())
    );
    assert!(
        !stats
            .last_effective_features
            .contains(&"clustered_lighting".to_string())
    );
    assert!(
        !stats
            .last_effective_features
            .contains(&"temporal".to_string())
    );
}

#[test]
fn render_framework_rejects_unknown_pipeline_handles() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    let error = server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(999))
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownPipeline { pipeline: 999 }
    );
}

#[test]
fn render_framework_rejects_quality_profile_when_requested_feature_lacks_backend_caps() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());

    let error = server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(true),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 0,
            reason:
                "quality profile `flagship` requires virtual_geometry, hybrid_global_illumination"
                    .to_string(),
            missing: missing_capabilities(&[
                RenderCapabilityKind::VirtualGeometry,
                RenderCapabilityKind::HybridGlobalIllumination,
            ]),
        }
    );
}

#[test]
fn render_framework_rejects_pipeline_switch_when_active_profile_loses_required_caps() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship").with_virtual_geometry(true),
        )
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());

    let error = server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 1,
            reason: "quality profile `flagship` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}

#[test]
#[cfg(target_os = "windows")]
fn render_framework_rejects_submit_when_active_profile_loses_required_caps() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("flagship").with_virtual_geometry(true),
        )
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());

    let error = server
        .submit_frame_extract(viewport, test_extract())
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 1,
            reason: "quality profile `flagship` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}

#[test]
fn render_framework_accepts_built_in_deferred_pipeline_handle() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(2)),
        "submit should honor the built-in deferred pipeline asset"
    );
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
}

#[test]
fn render_framework_registers_pipeline_assets_and_validates_reload() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(77);
    custom_pipeline.name = "custom-forward-plus".to_string();

    let handle = server.register_pipeline_asset(custom_pipeline).unwrap();
    server.reload_pipeline(handle).unwrap();
    server.set_pipeline_asset(viewport, handle).unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(handle, RenderPipelineHandle::new(77));
    assert_eq!(stats.last_pipeline, Some(RenderPipelineHandle::new(77)));
}

#[test]
fn registering_and_reloading_inactive_pipeline_assets_do_not_change_last_pipeline_stats() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let before = server.query_stats().unwrap().last_pipeline;
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(85);
    custom_pipeline.name = "inactive-custom-forward-plus".to_string();

    let handle = server.register_pipeline_asset(custom_pipeline).unwrap();

    assert_eq!(server.query_stats().unwrap().last_pipeline, before);
    server.reload_pipeline(handle).unwrap();
    assert_eq!(server.query_stats().unwrap().last_pipeline, before);
}

#[test]
fn render_framework_rejects_pipeline_asset_with_unknown_executor_id() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(78);
    custom_pipeline.name = "bad-executor-pipeline".to_string();
    let bloom = custom_pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-executor-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "bad-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.missing-executor")
                .with_side_effects(),
            ],
        ));

    let error = server.register_pipeline_asset(custom_pipeline).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::GraphCompileFailure {
            pipeline: 78,
            message:
                "render pass `bad-executor-pass` references unregistered executor `custom.missing-executor`"
                    .to_string(),
        }
    );
}

#[test]
fn render_framework_accepts_pipeline_asset_with_culled_unknown_executor_id() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(79);
    custom_pipeline.name = "bad-culled-executor-pipeline".to_string();
    let debug_overlay = custom_pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::DebugOverlay))
        .expect("default pipeline should include debug overlay");
    *debug_overlay = debug_overlay
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-culled-executor-feature",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Debug,
                    "bad-culled-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.culled-missing")
                .write_texture_with_schema(
                    "unused-custom-target",
                    crate::render_graph::RenderResourceSchema::texture(
                        crate::render_graph::RenderTextureSchema::new(
                            crate::rhi::TextureFormat::Rgba8Unorm,
                            crate::rhi::TextureUsage::RENDER_ATTACHMENT
                                | crate::rhi::TextureUsage::SAMPLED,
                        ),
                    ),
                ),
            ],
        ));

    let handle = server.register_pipeline_asset(custom_pipeline).unwrap();

    assert_eq!(handle, RenderPipelineHandle::new(79));
}

#[test]
fn render_framework_rejects_quality_gated_bad_descriptor_during_registration() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(80);
    custom_pipeline.name = "bad-gated-descriptor-pipeline".to_string();
    let bloom = custom_pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-descriptor",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Deferred,
                    "bad-gated-registration-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("post.uber"),
            ],
        ));

    let error = server.register_pipeline_asset(custom_pipeline).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::GraphCompileFailure {
            pipeline: 80,
            message:
                "feature descriptor `bad-gated-descriptor` pass `bad-gated-registration-pass` targets undeclared stage `Deferred`"
                    .to_string(),
        }
    );
}

#[test]
fn render_framework_rejects_quality_gated_unknown_executor_during_registration() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let mut custom_pipeline = RenderPipelineAsset::default_forward_plus();
    custom_pipeline.handle = RenderPipelineHandle::new(81);
    custom_pipeline.name = "bad-gated-executor-pipeline".to_string();
    let bloom = custom_pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-executor",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "bad-gated-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.gated-missing")
                .with_side_effects(),
            ],
        ));

    let error = server.register_pipeline_asset(custom_pipeline).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::GraphCompileFailure {
            pipeline: 81,
            message:
                "render pass `bad-gated-executor-pass` references unregistered executor `custom.gated-missing`"
                    .to_string(),
        }
    );
}

#[test]
fn quality_profile_can_override_the_default_pipeline_asset() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("deferred-quality")
                .with_pipeline_asset(RenderPipelineHandle::new(2)),
        )
        .unwrap();
    server
        .submit_frame_extract(viewport, test_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(
        stats.last_pipeline,
        Some(RenderPipelineHandle::new(2)),
        "quality profile pipeline override should become the viewport default when no explicit pipeline is set"
    );
    assert_eq!(stats.last_frame_history, Some(FrameHistoryHandle::new(1)));
}

#[test]
fn render_framework_rejects_quality_profile_with_unknown_pipeline_override() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    let error = server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("missing-override")
                .with_pipeline_asset(RenderPipelineHandle::new(404)),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownPipeline { pipeline: 404 }
    );
}

#[test]
fn render_framework_reports_profile_override_pipeline_for_capability_mismatch() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());

    let error = server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("deferred-vg")
                .with_pipeline_asset(RenderPipelineHandle::new(2))
                .with_virtual_geometry(true),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 2,
            reason: "quality profile `deferred-vg` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}

#[test]
fn render_framework_rejects_pipeline_switch_when_pipeline_asset_requires_missing_backend_caps() {
    let server = pluginized_wgpu_render_framework();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());
    let mut pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()]);
    pipeline.handle = RenderPipelineHandle::new(82);
    pipeline.name = "capability-gated-vg-pipeline".to_string();
    let handle = server.register_pipeline_asset(pipeline).unwrap();

    let error = server.set_pipeline_asset(viewport, handle).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 82,
            reason: "pipeline `capability-gated-vg-pipeline` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}

#[test]
fn render_framework_rejects_profile_override_when_pipeline_asset_requires_missing_backend_caps() {
    let server = pluginized_wgpu_render_framework();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server.override_capabilities_for_tests(capability_test_summary());
    let mut pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()]);
    pipeline.handle = RenderPipelineHandle::new(83);
    pipeline.name = "profile-override-vg-pipeline".to_string();
    let handle = server.register_pipeline_asset(pipeline).unwrap();

    let error = server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-pipeline-override").with_pipeline_asset(handle),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 83,
            reason: "pipeline `profile-override-vg-pipeline` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}

#[test]
fn render_framework_rejects_active_pipeline_reload_when_asset_requires_missing_backend_caps() {
    let server = pluginized_wgpu_render_framework();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    let mut base_pipeline = RenderPipelineAsset::default_forward_plus();
    base_pipeline.handle = RenderPipelineHandle::new(84);
    base_pipeline.name = "active-reload-base-pipeline".to_string();
    let handle = server.register_pipeline_asset(base_pipeline).unwrap();
    server.set_pipeline_asset(viewport, handle).unwrap();
    server.override_capabilities_for_tests(capability_test_summary());
    let mut reloaded_pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()]);
    reloaded_pipeline.handle = handle;
    reloaded_pipeline.name = "active-reload-vg-pipeline".to_string();
    server.register_pipeline_asset(reloaded_pipeline).unwrap();

    let error = server.reload_pipeline(handle).unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::CapabilityMismatch {
            pipeline: 84,
            reason: "pipeline `active-reload-vg-pipeline` requires virtual_geometry".to_string(),
            missing: missing_capabilities(&[RenderCapabilityKind::VirtualGeometry]),
        }
    );
}
