use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::GpuPassTimer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration, RuntimePrepareExternalBufferBinding,
    RuntimePrepareGpuPassProfile, RuntimePrepareGpuReadbackRequest,
};

pub(in crate::graphics::scene::scene_renderer::core) type SceneRendererRuntimePrepareCollector =
    Box<
        dyn FnMut(
                &wgpu::Device,
                &wgpu::Queue,
                &mut wgpu::CommandEncoder,
                &ResourceStreamer,
                &ViewportRenderFrame,
                &mut Vec<RuntimePrepareExternalBufferBinding>,
                &mut Vec<RuntimePrepareGpuReadbackRequest>,
                bool,
                Option<&mut GpuPassTimer>,
                &mut Vec<RuntimePrepareGpuPassProfile>,
            ) -> Result<RenderPluginRendererOutputs, GraphicsError>
            + Send,
    >;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginResources {
    capabilities: SceneRendererAdvancedPluginResourceCapabilities,
    runtime_prepare_collectors: Vec<SceneRendererRuntimePrepareCollector>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SceneRendererAdvancedPluginResourceCapabilities {
    virtual_geometry: bool,
    volumetric_fog: bool,
    #[cfg(test)]
    hybrid_gi: bool,
}

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        _device: &wgpu::Device,
        render_features: &[RenderFeatureDescriptor],
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Self {
        Self {
            capabilities: advanced_plugin_resource_capabilities(render_features),
            runtime_prepare_collectors: runtime_prepare_collectors
                .into_iter()
                .map(scene_runtime_prepare_collector_from_registration)
                .collect(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn register_runtime_prepare_collector(
        &mut self,
        collector: SceneRendererRuntimePrepareCollector,
    ) {
        self.runtime_prepare_collectors.push(collector);
    }

    pub(super) fn runtime_prepare_collectors_mut(
        &mut self,
    ) -> &mut [SceneRendererRuntimePrepareCollector] {
        &mut self.runtime_prepare_collectors
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_enabled(
        &self,
    ) -> bool {
        self.capabilities.virtual_geometry
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn volumetric_fog_enabled(&self) -> bool {
        self.capabilities.volumetric_fog
    }

    #[cfg(test)]
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
        volumetric_fog: render_features
            .iter()
            .any(|feature| feature.name == "volumetric_fog"),
        #[cfg(test)]
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

fn scene_runtime_prepare_collector_from_registration(
    registration: RuntimePrepareCollectorRegistration,
) -> SceneRendererRuntimePrepareCollector {
    Box::new(
        move |device,
              queue,
              encoder,
              streamer,
              frame,
              external_buffer_bindings,
              gpu_readbacks,
              gpu_work_admitted,
              gpu_pass_timer,
              gpu_pass_profiles| {
            let mut context =
                RuntimePrepareCollectorContext::new_with_gpu_readbacks_and_gpu_work_admission(
                    device,
                    queue,
                    encoder,
                    streamer,
                    frame,
                    external_buffer_bindings,
                    gpu_readbacks,
                    gpu_work_admitted,
                    gpu_pass_timer,
                    gpu_pass_profiles,
                );
            registration.collect(&mut context)
        },
    )
}

#[cfg(test)]
mod tests {
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
            .with_capability_requirement(
                RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
            ),
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
}
