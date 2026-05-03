use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};

pub(in crate::graphics::scene::scene_renderer::core) type SceneRendererRuntimePrepareCollector =
    Box<
        dyn Fn(
                &wgpu::Device,
                &wgpu::Queue,
                &mut wgpu::CommandEncoder,
                &ResourceStreamer,
                &ViewportRenderFrame,
            ) -> Result<RenderPluginRendererOutputs, GraphicsError>
            + Send
            + Sync,
    >;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginResources {
    capabilities: SceneRendererAdvancedPluginResourceCapabilities,
    runtime_prepare_collectors: Vec<SceneRendererRuntimePrepareCollector>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SceneRendererAdvancedPluginResourceCapabilities {
    virtual_geometry: bool,
    hybrid_gi: bool,
}

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        _device: &wgpu::Device,
        render_features: &[RenderFeatureDescriptor],
    ) -> Self {
        Self {
            capabilities: advanced_plugin_resource_capabilities(render_features),
            runtime_prepare_collectors: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn new_with_virtual_geometry_for_test(
        _device: &wgpu::Device,
    ) -> Self {
        Self {
            capabilities: SceneRendererAdvancedPluginResourceCapabilities {
                virtual_geometry: true,
                ..SceneRendererAdvancedPluginResourceCapabilities::default()
            },
            runtime_prepare_collectors: Vec::new(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn register_runtime_prepare_collector(
        &mut self,
        collector: SceneRendererRuntimePrepareCollector,
    ) {
        self.runtime_prepare_collectors.push(collector);
    }

    pub(super) fn runtime_prepare_collectors(&self) -> &[SceneRendererRuntimePrepareCollector] {
        &self.runtime_prepare_collectors
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_enabled(
        &self,
    ) -> bool {
        self.capabilities.virtual_geometry
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn hybrid_gi_enabled(&self) -> bool {
        self.capabilities.hybrid_gi
    }
}

fn advanced_plugin_resource_capabilities(
    render_features: &[RenderFeatureDescriptor],
) -> SceneRendererAdvancedPluginResourceCapabilities {
    SceneRendererAdvancedPluginResourceCapabilities {
        virtual_geometry: render_features_require(
            render_features,
            RenderFeatureCapabilityRequirement::VirtualGeometry,
        ),
        hybrid_gi: render_features_require(
            render_features,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ),
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
            .with_capability_requirement(
                RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
            ),
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
        assert_eq!(
            capabilities,
            SceneRendererAdvancedPluginResourceCapabilities {
                virtual_geometry: true,
                hybrid_gi: true,
            }
        );
    }
}
