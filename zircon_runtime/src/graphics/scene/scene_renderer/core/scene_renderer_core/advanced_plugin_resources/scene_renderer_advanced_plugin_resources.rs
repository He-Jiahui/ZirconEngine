use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::GpuPassTimer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RuntimePrepareCollectorContext,
    RuntimePrepareCollectorRegistration, RuntimePrepareDeviceEpoch,
    RuntimePrepareExternalBufferBinding, RuntimePrepareFramePacket, RuntimePrepareGpuPassProfile,
    RuntimePrepareGpuReadbackRequest,
};

pub(in crate::graphics::scene::scene_renderer::core) type SceneRendererRuntimePrepareCollector =
    Box<
        dyn FnMut(
                &wgpu::Device,
                RuntimePrepareDeviceEpoch,
                &mut wgpu::CommandEncoder,
                &ResourceStreamer,
                &ViewportRenderFrame,
                &mut Vec<RuntimePrepareExternalBufferBinding>,
                &mut Vec<RuntimePrepareGpuReadbackRequest>,
                bool,
                Option<&mut GpuPassTimer>,
                &mut Vec<RuntimePrepareGpuPassProfile>,
                &mut RuntimePrepareFramePacket,
            ) -> Result<RenderPluginRendererOutputs, GraphicsError>
            + Send,
    >;

/// 将注册表中的 CPU 能力声明和 runtime-prepare 回调固定到 renderer 世代。
/// GPU 权限仅在回调执行时经上下文授予，构造阶段不借用原生 device。
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginResources {
    capabilities: SceneRendererAdvancedPluginResourceCapabilities,
    runtime_prepare_collectors: Vec<SceneRendererRuntimePrepareCollector>,
    runtime_prepare_gpu_readback_collector_count: usize,
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
        render_features: &[RenderFeatureDescriptor],
        runtime_prepare_collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    ) -> Self {
        let runtime_prepare_collectors = runtime_prepare_collectors.into_iter().collect::<Vec<_>>();
        let runtime_prepare_gpu_readback_collector_count = runtime_prepare_collectors
            .iter()
            .filter(|registration| registration.requests_gpu_readback())
            .count();
        Self {
            capabilities: advanced_plugin_resource_capabilities(render_features),
            runtime_prepare_collectors: runtime_prepare_collectors
                .into_iter()
                .map(scene_runtime_prepare_collector_from_registration)
                .collect(),
            runtime_prepare_gpu_readback_collector_count,
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

    pub(in crate::graphics::scene::scene_renderer::core) fn has_runtime_prepare_gpu_readback_collectors(
        &self,
    ) -> bool {
        self.runtime_prepare_gpu_readback_collector_count > 0
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

/// 把插件登记转换为帧回调；回调产生的上传 packet 交给场景帧事务，而非插件自行提交。
fn scene_runtime_prepare_collector_from_registration(
    registration: RuntimePrepareCollectorRegistration,
) -> SceneRendererRuntimePrepareCollector {
    Box::new(
        move |device,
              device_epoch,
              encoder,
              streamer,
              frame,
              external_buffer_bindings,
              gpu_readbacks,
              gpu_work_admitted,
              gpu_pass_timer,
              gpu_pass_profiles,
              runtime_prepare_frame_packet| {
            let mut context =
                RuntimePrepareCollectorContext::new_with_gpu_readbacks_and_gpu_work_admission(
                    device,
                    device_epoch,
                    encoder,
                    streamer,
                    frame,
                    external_buffer_bindings,
                    gpu_readbacks,
                    gpu_work_admitted,
                    gpu_pass_timer,
                    gpu_pass_profiles,
                );
            let result = registration.collect(&mut context);
            let mut collector_frame_packet = context.take_frame_packet();
            runtime_prepare_frame_packet.append(&mut collector_frame_packet);
            result
        },
    )
}

#[cfg(test)]
#[path = "tests/scene_renderer_advanced_plugin_resources.rs"]
mod tests;
