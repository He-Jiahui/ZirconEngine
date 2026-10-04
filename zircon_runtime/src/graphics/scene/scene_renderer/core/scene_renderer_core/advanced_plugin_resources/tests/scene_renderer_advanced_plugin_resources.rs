use super::*;

#[test]
fn resource_capability_scan_ignores_unqualified_descriptors() {
    let render_features = vec![RenderFeatureDescriptor::new(
        "fallback-virtual-geometry-without-resource-capability",
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )];
    let capabilities = advanced_plugin_resource_capabilities(&render_features);

    assert!(!render_features_require(
        &render_features,
        RenderFeatureCapabilityRequirement::VirtualGeometry
    ));
    assert!(!render_features_require(
        &render_features,
        RenderFeatureCapabilityRequirement::HybridGlobalIllumination
    ));
    assert_eq!(
        capabilities,
        SceneRendererAdvancedPluginResourceCapabilities::default()
    );
}

#[test]
fn resource_capability_scan_accepts_advanced_plugin_descriptors() {
    let render_features = vec![
        RenderFeatureDescriptor::new(
            "plugin.virtual_geometry.resources",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
        RenderFeatureDescriptor::new(
            "plugin.hybrid_gi.resources",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination),
        RenderFeatureDescriptor::new("volumetric_fog", Vec::new(), Vec::new(), Vec::new()),
    ];
    let capabilities = advanced_plugin_resource_capabilities(&render_features);

    assert!(render_features_require(
        &render_features,
        RenderFeatureCapabilityRequirement::VirtualGeometry
    ));
    assert!(render_features_require(
        &render_features,
        RenderFeatureCapabilityRequirement::HybridGlobalIllumination
    ));
    let resources = SceneRendererAdvancedPluginResources {
        capabilities,
        runtime_prepare_collectors: Vec::new(),
        runtime_prepare_gpu_readback_collector_count: 0,
    };
    assert!(resources.hybrid_gi_enabled());
    assert!(resources.volumetric_fog_enabled());
    assert_eq!(
        capabilities,
        SceneRendererAdvancedPluginResourceCapabilities {
            virtual_geometry: true,
            volumetric_fog: true,
            hybrid_gi: true,
        }
    );
}
