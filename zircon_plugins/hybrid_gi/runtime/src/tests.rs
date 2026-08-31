use super::*;

#[test]
fn transient_handoff_consumers_preserve_compiler_buffer_windows() {
    let scene_depth = include_str!("render_pass_executors/scene_depth_handoff.rs");
    let trace_schedule = include_str!("render_pass_executors/trace_schedule_handoff.rs");
    let resolve = include_str!("render_pass_executors/resolve_trace_handoff.rs");
    let scene_depth_production = scene_depth
        .split_once("#[cfg(test)]")
        .map(|(source, _)| source)
        .expect("scene depth handoff test boundary");
    let trace_schedule_production = trace_schedule
        .split_once("#[cfg(test)]")
        .map(|(source, _)| source)
        .expect("trace schedule handoff test boundary");
    let resolve_production = resolve
        .split_once("#[cfg(test)]")
        .map(|(source, _)| source)
        .expect("resolve handoff test boundary");

    assert!(scene_depth.contains("require_buffer_binding("));
    assert!(scene_depth.contains("gpu.native_context()"));
    assert!(scene_depth.contains("drop(native)"));
    assert!(scene_depth.contains("RenderPassGpuResourceFactory"));
    assert!(scene_depth.contains("gpu.plugin_outputs().hybrid_gi.scene_prepare"));
    assert!(!scene_depth_production.contains("gpu.device"));
    assert!(!scene_depth_production.contains("gpu.encoder"));
    assert!(!scene_depth_production.contains("native.device"));
    assert!(scene_depth.contains("write_buffer_binding("));
    assert!(scene_depth.contains("BindingResource::Buffer(hybrid_gi_scene_buffer)"));
    assert!(scene_depth.contains("binding.buffer.size().checked_sub(binding.offset)"));
    assert!(!scene_depth.contains("hybrid_gi_scene_buffer.as_entire_binding()"));
    assert!(trace_schedule.contains("require_buffer_binding("));
    assert!(trace_schedule.contains("gpu.native_context()"));
    assert!(trace_schedule.contains("drop(native)"));
    assert!(trace_schedule.contains("RenderPassGpuResourceFactory"));
    assert!(!trace_schedule_production.contains("gpu.device"));
    assert!(!trace_schedule_production.contains("gpu.encoder"));
    assert!(!trace_schedule_production.contains("native.device"));
    assert!(trace_schedule.contains("BindingResource::Buffer(hybrid_gi_scene_buffer)"));
    assert!(trace_schedule.contains("BindingResource::Buffer(hybrid_gi_trace_buffer)"));
    assert!(!trace_schedule.contains("hybrid_gi_scene_buffer.as_entire_binding()"));
    assert!(!trace_schedule.contains("hybrid_gi_trace_buffer.as_entire_binding()"));
    assert!(resolve.contains("require_buffer_binding("));
    assert!(resolve.contains("gpu.native_context()"));
    assert!(resolve.contains("drop(native)"));
    assert!(resolve.contains("RenderPassGpuResourceFactory"));
    assert!(!resolve_production.contains("gpu.device"));
    assert!(!resolve_production.contains("gpu.encoder"));
    assert!(!resolve_production.contains("native.device"));
    assert!(resolve.contains("BindingResource::Buffer(hybrid_gi_trace_buffer)"));
    assert!(!resolve.contains("hybrid_gi_trace_buffer.as_entire_binding()"));
}

#[test]
fn hybrid_gi_registration_contributes_render_feature_descriptor() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(
        report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == HYBRID_GI_MODULE_NAME)
    );
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
    assert!(
        report
            .package_manifest
            .capabilities
            .contains(&HYBRID_GI_ADVANCED_RENDER_CAPABILITY.to_string())
    );
    assert!(
        report.package_manifest.modules[0]
            .capabilities
            .contains(&HYBRID_GI_ADVANCED_RENDER_CAPABILITY.to_string())
    );
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
    assert!(resolve_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::SCENE_VELOCITY
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Read
    }));
    for resource_name in [
        zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
        zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
    ] {
        let resource = resolve_pass
            .resources
            .iter()
            .find(|resource| resource.name == resource_name)
            .expect("hybrid GI history resource");
        assert_eq!(
            resource.access,
            zircon_runtime::graphics::RenderFeatureResourceAccess::Read
        );
        assert!(resource.usage.persistent);
        assert_eq!(resource.schema, Some(super::hybrid_gi_history_schema()));
        assert_eq!(
            resource.access_metadata,
            Some(
                zircon_runtime::render_graph::RenderGraphResourceAccessMetadata::new(
                    zircon_runtime::render_graph::RenderGraphResourceAccessRange::Texture(
                        zircon_runtime::render_graph::RenderGraphTextureSubresourceRange::full(),
                    ),
                    zircon_runtime::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
                        zircon_runtime::render_graph::RenderGraphShaderStages::FRAGMENT,
                    ),
                ),
            )
        );
    }
    assert!(resolve_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Write
            && resource.usage.persistent
    }));
    assert!(resolve_pass.resources.iter().any(|resource| {
        resource.name
            == zircon_runtime::core::framework::render::PostProcessGraphResourceNames::HYBRID_GI_LIGHTING
            && resource.access == zircon_runtime::graphics::RenderFeatureResourceAccess::Write
            && resource.usage.persistent
    }));
    assert_eq!(report.extensions.render_pass_executors().len(), 3);
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
