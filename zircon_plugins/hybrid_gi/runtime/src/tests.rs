use super::*;

#[test]
fn hybrid_gi_registration_contributes_render_feature_descriptor() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == HYBRID_GI_MODULE_NAME));
    assert_eq!(
        report.extensions.render_features()[0].name,
        HYBRID_GI_FEATURE_NAME
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "rendering");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert!(report
        .package_manifest
        .capabilities
        .contains(&HYBRID_GI_ADVANCED_RENDER_CAPABILITY.to_string()));
    assert!(report.package_manifest.modules[0]
        .capabilities
        .contains(&HYBRID_GI_ADVANCED_RENDER_CAPABILITY.to_string()));
    let feature = &report.extensions.render_features()[0];
    assert_eq!(
        feature.required_extract_sections,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string()
        ]
    );
    assert_eq!(
        feature.capability_requirements,
        vec![
            zircon_runtime::graphics::RenderFeatureCapabilityRequirement::HybridGlobalIllumination
        ]
    );
    assert_eq!(
        feature.history_bindings,
        vec![zircon_runtime::graphics::FrameHistoryBinding::read_write(
            zircon_runtime::graphics::FrameHistorySlot::GlobalIllumination
        )]
    );
    assert_eq!(
        feature
            .stage_passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "hybrid-gi-scene-prepare",
            "hybrid-gi-trace-schedule",
            "hybrid-gi-resolve",
            "hybrid-gi-history",
        ]
    );
    let scene_prepare_pass = feature
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hybrid-gi-scene-prepare")
        .expect("hybrid GI scene prepare pass");
    assert_eq!(
        scene_prepare_pass.queue,
        zircon_runtime::render_graph::QueueLane::AsyncCompute
    );
    let scene_depth_handoff_workload = scene_prepare_pass.compute_workload.as_ref().expect(
        "hybrid GI scene prepare pass should declare graph depth handoff workload metadata",
    );
    assert_eq!(
        scene_depth_handoff_workload.pipeline_label,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL
    );
    assert_eq!(
        scene_depth_handoff_workload.workgroup_size,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE
    );
    assert_eq!(
        scene_depth_handoff_workload.dispatch_extent,
        zircon_runtime::render_graph::RenderGraphComputeDispatchExtent::Fixed(
            HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS
        )
    );
    assert!(scene_prepare_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HZB_FURTHEST
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Read
    }));
    assert!(scene_prepare_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_SCENE
            && resource.minimum_size_bytes == Some(HYBRID_GI_SCENE_BUFFER_MINIMUM_SIZE_BYTES)
    }));
    let trace_schedule_pass = feature
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hybrid-gi-trace-schedule")
        .expect("hybrid GI trace schedule pass");
    assert_eq!(
        trace_schedule_pass.queue,
        zircon_runtime::render_graph::QueueLane::AsyncCompute
    );
    let trace_workload = trace_schedule_pass
        .compute_workload
        .as_ref()
        .expect("hybrid GI trace schedule pass should declare compute workload metadata");
    assert_eq!(
        trace_workload.pipeline_label,
        "zircon-hybrid-gi-trace-schedule"
    );
    assert_eq!(trace_workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        trace_workload.dispatch_extent,
        zircon_runtime::render_graph::RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
    assert!(trace_schedule_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_TRACE
            && resource.minimum_size_bytes == Some(HYBRID_GI_TRACE_BUFFER_MINIMUM_SIZE_BYTES)
    }));
    let resolve_pass = feature
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hybrid-gi-resolve")
        .expect("hybrid GI temporal resolve pass");
    for resource_name in [
        zircon_runtime::core::framework::render::PostProcessGraphResourceNames::SCENE_VELOCITY,
        zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
        zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
    ] {
        assert!(resolve_pass.resources.iter().any(|resource| {
            resource.name == resource_name
                && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Read
        }));
    }
    assert!(resolve_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Write
    }));
    let history_pass = feature
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "hybrid-gi-history")
        .expect("hybrid GI history pass");
    assert!(history_pass
        .resources
        .iter()
        .any(|resource| resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_LIGHTING
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Read));
    assert!(history_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Write
    }));
    assert_eq!(report.extensions.render_pass_executors().len(), 4);
    assert_eq!(report.extensions.runtime_prepare_collectors().len(), 1);
    assert_eq!(
        report.extensions.runtime_prepare_collectors()[0].collector_id(),
        "hybrid-gi.runtime-prepare"
    );
    assert_eq!(
        report.extensions.hybrid_gi_runtime_providers()[0].provider_id(),
        "plugin.hybrid_gi.runtime"
    );
    assert_eq!(
        report
            .extensions
            .render_pass_executors()
            .iter()
            .map(|registration| registration.executor_id().as_str())
            .collect::<Vec<_>>(),
        vec![
            "hybrid-gi.scene-prepare",
            "hybrid-gi.trace-schedule",
            "hybrid-gi.resolve",
            "hybrid-gi.history",
        ]
    );
}

#[test]
fn hybrid_gi_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("hybrid_gi distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, "zircon_plugin_hybrid_gi_dist");
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        "zircon_plugin_hybrid_gi_runtime_entry_v3"
    );

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "hybrid_gi.dist")
        .expect("hybrid_gi native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, "zircon_plugin_hybrid_gi_dist");
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
