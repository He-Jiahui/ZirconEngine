use super::super::super::super::mesh::VirtualGeometryIndirectArgsGpuResources;
use crate::graphics::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginResources;

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        _device: &wgpu::Device,
        render_features: &[RenderFeatureDescriptor],
    ) -> Self {
        let _ = render_features_require(
            render_features,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        );
        let _ = render_features_require(
            render_features,
            RenderFeatureCapabilityRequirement::VirtualGeometry,
        );
        Self
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_virtual_geometry_for_test(
        _device: &wgpu::Device,
    ) -> Self {
        Self
    }

    pub(super) fn virtual_geometry_indirect_args(
        &self,
    ) -> Option<&VirtualGeometryIndirectArgsGpuResources> {
        None
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
