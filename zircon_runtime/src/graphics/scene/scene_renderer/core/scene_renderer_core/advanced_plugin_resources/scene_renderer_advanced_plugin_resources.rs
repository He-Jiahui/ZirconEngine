use super::super::super::super::hybrid_gi::HybridGiGpuResources;
use super::super::super::super::mesh::VirtualGeometryIndirectArgsGpuResources;
use super::super::super::super::virtual_geometry::VirtualGeometryGpuResources;
use crate::graphics::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginResources {
    hybrid_gi: Option<HybridGiGpuResources>,
    virtual_geometry: Option<VirtualGeometryGpuResources>,
    virtual_geometry_indirect_args: Option<VirtualGeometryIndirectArgsGpuResources>,
}

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        device: &wgpu::Device,
        render_features: &[RenderFeatureDescriptor],
    ) -> Self {
        Self::with_enabled_resources(
            device,
            render_features_require(
                render_features,
                RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
            ),
            render_features_require(
                render_features,
                RenderFeatureCapabilityRequirement::VirtualGeometry,
            ),
        )
    }

    fn with_enabled_resources(
        device: &wgpu::Device,
        hybrid_gi_enabled: bool,
        virtual_geometry_enabled: bool,
    ) -> Self {
        Self {
            hybrid_gi: hybrid_gi_enabled.then(|| HybridGiGpuResources::new(device)),
            virtual_geometry: virtual_geometry_enabled
                .then(|| VirtualGeometryGpuResources::new(device)),
            virtual_geometry_indirect_args: virtual_geometry_enabled
                .then(|| VirtualGeometryIndirectArgsGpuResources::new(device)),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_virtual_geometry_for_test(
        device: &wgpu::Device,
    ) -> Self {
        Self::with_enabled_resources(device, false, true)
    }

    pub(super) fn hybrid_gi(&self) -> Option<&HybridGiGpuResources> {
        self.hybrid_gi.as_ref()
    }

    pub(super) fn virtual_geometry(&self) -> Option<&VirtualGeometryGpuResources> {
        self.virtual_geometry.as_ref()
    }

    pub(super) fn virtual_geometry_indirect_args(
        &self,
    ) -> Option<&VirtualGeometryIndirectArgsGpuResources> {
        self.virtual_geometry_indirect_args.as_ref()
    }
}

fn render_features_require(
    render_features: &[RenderFeatureDescriptor],
    requirement: RenderFeatureCapabilityRequirement,
) -> bool {
    render_features
        .iter()
        .any(|feature| feature.capability_requirements.contains(&requirement))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_capability_scan_ignores_unqualified_descriptors() {
        let render_features = vec![RenderFeatureDescriptor::new(
            "legacy-virtual-geometry-without-resource-capability",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )];

        assert!(!render_features_require(
            &render_features,
            RenderFeatureCapabilityRequirement::VirtualGeometry
        ));
        assert!(!render_features_require(
            &render_features,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination
        ));
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
            .with_capability_requirement(
                RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
            ),
        ];

        assert!(render_features_require(
            &render_features,
            RenderFeatureCapabilityRequirement::VirtualGeometry
        ));
        assert!(render_features_require(
            &render_features,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination
        ));
    }
}
