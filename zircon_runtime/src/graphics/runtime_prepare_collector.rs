use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, ViewportCameraSnapshot,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
    RenderPreparedRuntimeSidebands, RenderSceneSnapshot, RenderVirtualGeometryReadbackOutputs,
};
use crate::core::math::UVec2;
#[cfg(test)]
use crate::core::math::Vec4;
use crate::graphics::GraphicsError;
use crate::graphics::ViewportRenderFrame;

pub trait RuntimePrepareCollector: Send + Sync {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError>;
}

pub struct RuntimePrepareCollectorContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub frame_extract: &'a RenderFrameExtract,
    frame: &'a ViewportRenderFrame,
    external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
}

impl<'a> RuntimePrepareCollectorContext<'a> {
    pub(crate) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame_extract: &frame.extract,
            frame,
            external_buffer_bindings,
        }
    }

    pub fn frame_extract(&self) -> &RenderFrameExtract {
        self.frame_extract
    }

    pub fn scene_snapshot(&self) -> &RenderSceneSnapshot {
        &self.frame.scene
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }

    pub fn prepared_runtime_sidebands(&self) -> &RenderPreparedRuntimeSidebands {
        self.frame.prepared_runtime_sidebands()
    }

    pub fn prepared_plugin_renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.prepared_runtime_sidebands().plugin_renderer_outputs
    }

    pub fn prepared_hybrid_gi_readback_outputs(&self) -> &RenderHybridGiReadbackOutputs {
        self.prepared_runtime_sidebands()
            .hybrid_gi_readback_outputs()
    }

    pub fn prepared_virtual_geometry_readback_outputs(
        &self,
    ) -> &RenderVirtualGeometryReadbackOutputs {
        self.prepared_runtime_sidebands()
            .virtual_geometry_readback_outputs()
    }

    pub fn prepared_hybrid_gi_evictable_probe_ids(&self) -> &[u32] {
        self.prepared_runtime_sidebands()
            .hybrid_gi_evictable_probe_ids()
    }

    pub fn prepared_virtual_geometry_evictable_page_ids(&self) -> &[u32] {
        self.prepared_runtime_sidebands()
            .virtual_geometry_evictable_page_ids()
    }

    pub fn register_external_buffer_binding(
        &mut self,
        logical_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) {
        let logical_name = logical_name.into();
        let backing_name = format!("{logical_name}:runtime-prepare");
        self.register_external_buffer_binding_with_backing(logical_name, backing_name, buffer);
    }

    pub fn register_external_buffer_binding_with_backing(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) {
        self.external_buffer_bindings
            .push(RuntimePrepareExternalBufferBinding::new(
                logical_name,
                backing_name,
                buffer,
            ));
    }
}

#[derive(Clone)]
pub(crate) struct RuntimePrepareExternalBufferBinding {
    logical_name: String,
    backing_name: String,
    buffer: wgpu::Buffer,
}

impl RuntimePrepareExternalBufferBinding {
    pub(crate) fn new(
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) -> Self {
        Self {
            logical_name: logical_name.into(),
            backing_name: backing_name.into(),
            buffer: buffer.clone(),
        }
    }

    pub(crate) fn logical_name(&self) -> &str {
        &self.logical_name
    }

    pub(crate) fn backing_name(&self) -> &str {
        &self.backing_name
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

struct FunctionRuntimePrepareCollector {
    collector: RuntimePrepareCollectorFn,
}

impl RuntimePrepareCollector for FunctionRuntimePrepareCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        (self.collector)(context)
    }
}

pub type RuntimePrepareCollectorFn = fn(
    &mut RuntimePrepareCollectorContext<'_>,
) -> Result<RenderPluginRendererOutputs, GraphicsError>;

#[derive(Clone)]
pub struct RuntimePrepareCollectorRegistration {
    collector_id: String,
    collector: Arc<dyn RuntimePrepareCollector>,
}

impl RuntimePrepareCollectorRegistration {
    pub fn new(collector_id: impl Into<String>, collector: RuntimePrepareCollectorFn) -> Self {
        Self::new_collector(
            collector_id,
            Arc::new(FunctionRuntimePrepareCollector { collector }),
        )
    }

    pub fn new_collector(
        collector_id: impl Into<String>,
        collector: Arc<dyn RuntimePrepareCollector>,
    ) -> Self {
        Self {
            collector_id: collector_id.into(),
            collector,
        }
    }

    pub fn collector_id(&self) -> &str {
        &self.collector_id
    }

    pub fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        self.collector.collect(context)
    }
}

impl fmt::Debug for RuntimePrepareCollectorRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimePrepareCollectorRegistration")
            .field("collector_id", &self.collector_id)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderWorldSnapshotHandle,
    };
    use crate::graphics::backend::RenderBackend;

    #[test]
    fn collector_context_exposes_viewport_size_extract_and_prepared_sidebands() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let RenderBackend { device, queue, .. } = backend;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-prepare-context-test-encoder"),
        });
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(44),
            empty_scene_snapshot(),
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720))
            .with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
                RenderPluginRendererOutputs {
                    virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                        node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                            page_request_ids: vec![300],
                            ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                        },
                        ..RenderVirtualGeometryReadbackOutputs::default()
                    },
                    hybrid_gi: RenderHybridGiReadbackOutputs {
                        completed_probe_ids: vec![7],
                        ..RenderHybridGiReadbackOutputs::default()
                    },
                    ..RenderPluginRendererOutputs::default()
                },
                vec![11],
                vec![22],
            ));

        let mut external_buffer_bindings = Vec::new();
        let context = RuntimePrepareCollectorContext::new(
            &device,
            &queue,
            &mut encoder,
            &frame,
            &mut external_buffer_bindings,
        );

        assert_eq!(context.viewport_size(), UVec2::new(1280, 720));
        assert_eq!(context.frame_extract().world.raw(), 44);
        assert_eq!(context.scene_snapshot().scene.meshes.len(), 0);
        assert_eq!(
            context
                .prepared_hybrid_gi_readback_outputs()
                .completed_probe_ids,
            vec![7]
        );
        assert_eq!(
            context
                .prepared_virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
        assert_eq!(context.prepared_hybrid_gi_evictable_probe_ids(), &[11]);
        assert_eq!(
            context.prepared_virtual_geometry_evictable_page_ids(),
            &[22]
        );
    }

    #[test]
    fn collector_context_registers_external_buffer_bindings() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let RenderBackend { device, queue, .. } = backend;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-prepare-context-buffer-binding-test-encoder"),
        });
        let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(64, 64));
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-runtime-prepare-context-external-buffer"),
            size: 32,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut external_buffer_bindings = Vec::new();

        {
            let mut context = RuntimePrepareCollectorContext::new(
                &device,
                &queue,
                &mut encoder,
                &frame,
                &mut external_buffer_bindings,
            );
            context.register_external_buffer_binding_with_backing(
                "particles.gpu.counters",
                "particles.gpu.counters:test-runtime-prepare",
                &buffer,
            );
        }

        assert_eq!(external_buffer_bindings.len(), 1);
        assert_eq!(
            external_buffer_bindings[0].logical_name(),
            "particles.gpu.counters"
        );
        assert_eq!(
            external_buffer_bindings[0].backing_name(),
            "particles.gpu.counters:test-runtime-prepare"
        );
    }

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }
}
