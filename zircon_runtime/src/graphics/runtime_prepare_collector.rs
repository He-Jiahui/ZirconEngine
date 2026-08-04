use std::fmt;
use std::ops::Range;
use std::sync::{Arc, Mutex};

#[cfg(test)]
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, ViewportCameraSnapshot,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
    RenderPreparedRuntimeSidebands, RenderSceneSnapshot, RenderVirtualGeometryReadbackOutputs,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::MaterialCaptureSeed;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::GraphicsError;
use crate::graphics::ViewportRenderFrame;
use zr_rhi_wgpu::{GpuReadbackQueue, ReadbackError};

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
    streamer: &'a ResourceStreamer,
    external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
    gpu_readbacks: Option<&'a mut Vec<RuntimePrepareGpuReadbackRequest>>,
}

impl<'a> RuntimePrepareCollectorContext<'a> {
    pub(crate) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        streamer: &'a ResourceStreamer,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: None,
        }
    }

    pub(crate) fn new_with_gpu_readbacks(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        streamer: &'a ResourceStreamer,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
        gpu_readbacks: &'a mut Vec<RuntimePrepareGpuReadbackRequest>,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: Some(gpu_readbacks),
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

    pub fn material_capture_seed(
        &self,
        id: &ResourceId,
    ) -> Option<RuntimePrepareMaterialCaptureSeed> {
        self.streamer
            .material_capture_seed(id)
            .map(RuntimePrepareMaterialCaptureSeed::from_material_capture_seed)
    }

    pub fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4> {
        self.streamer.sample_texture_rgba(id, uv)
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

    pub fn request_gpu_readback(
        &mut self,
        name: impl Into<String>,
        buffer: &wgpu::Buffer,
        range: Range<u64>,
    ) -> Result<RuntimeGpuReadback, GraphicsError> {
        let requests = self.gpu_readbacks.as_deref_mut().ok_or_else(|| {
            GraphicsError::BufferMap(
                "runtime prepare collector has no GPU readback request sink".to_owned(),
            )
        })?;
        let readback = RuntimeGpuReadback::pending();
        requests.push(RuntimePrepareGpuReadbackRequest {
            name: name.into(),
            buffer: buffer.clone(),
            range,
            completion: Arc::clone(&readback.completion),
        });
        Ok(readback)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeGpuReadback {
    completion: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
}

impl RuntimeGpuReadback {
    fn pending() -> Self {
        Self {
            completion: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.completion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    pub fn try_take(&self) -> Option<Result<Vec<u8>, GraphicsError>> {
        self.completion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .map(|result| result.map_err(GraphicsError::BufferMap))
    }
}

pub(crate) struct RuntimePrepareGpuReadbackRequest {
    name: String,
    buffer: wgpu::Buffer,
    range: Range<u64>,
    completion: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
}

impl RuntimePrepareGpuReadbackRequest {
    pub(crate) fn register(self, queue: &mut GpuReadbackQueue) -> Result<(), ReadbackError> {
        let completion = Arc::clone(&self.completion);
        let result = queue.request_readback_external(
            self.name,
            &self.buffer,
            self.range,
            Box::new(move |result| {
                *completion
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(
                    result
                        .map(<[u8]>::to_vec)
                        .map_err(|error| error.to_string()),
                );
            }),
        );
        if let Err(error) = &result {
            *self
                .completion
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Err(error.to_string()));
        }
        result.map(|_| ())
    }

    pub(crate) fn fail(self, error: impl Into<String>) {
        *self
            .completion
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Err(error.into()));
    }
}

#[derive(Clone, Debug)]
pub struct RuntimePrepareMaterialCaptureSeed {
    pub base_color: Vec4,
    pub emissive: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub double_sided: bool,
    pub alpha_blend: bool,
    pub alpha_cutoff: Option<f32>,
    pub base_color_texture: Option<ResourceId>,
    pub normal_texture: Option<ResourceId>,
    pub metallic_roughness_texture: Option<ResourceId>,
    pub occlusion_texture: Option<ResourceId>,
    pub emissive_texture: Option<ResourceId>,
}

impl RuntimePrepareMaterialCaptureSeed {
    fn from_material_capture_seed(seed: MaterialCaptureSeed) -> Self {
        Self {
            base_color: seed.base_color,
            emissive: seed.emissive,
            metallic: seed.metallic,
            roughness: seed.roughness,
            double_sided: seed.double_sided,
            alpha_blend: seed.alpha_blend,
            alpha_cutoff: seed.alpha_cutoff,
            base_color_texture: seed.base_color_texture,
            normal_texture: seed.normal_texture,
            metallic_roughness_texture: seed.metallic_roughness_texture,
            occlusion_texture: seed.occlusion_texture,
            emissive_texture: seed.emissive_texture,
        }
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
        let streamer = test_resource_streamer(&device, &queue);
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
            &streamer,
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
        let streamer = test_resource_streamer(&device, &queue);
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
                &streamer,
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

    #[test]
    fn collector_context_returns_nonblocking_shared_queue_readback_handles() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let RenderBackend { device, queue, .. } = backend;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-prepare-context-readback-test-encoder"),
        });
        let streamer = test_resource_streamer(&device, &queue);
        let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-runtime-prepare-context-readback-source"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let mut external_buffer_bindings = Vec::new();
        let mut gpu_readbacks = Vec::new();
        let mut context = RuntimePrepareCollectorContext::new_with_gpu_readbacks(
            &device,
            &queue,
            &mut encoder,
            &streamer,
            &frame,
            &mut external_buffer_bindings,
            &mut gpu_readbacks,
        );

        let readback = context
            .request_gpu_readback("test.runtime-prepare", &buffer, 0..4)
            .unwrap();

        assert!(!readback.is_ready());
        assert_eq!(gpu_readbacks.len(), 1);
        gpu_readbacks.pop().unwrap().fail("test readback rejection");
        assert!(readback.is_ready());
        assert!(readback.try_take().unwrap().is_err());
    }

    #[test]
    fn collector_context_exposes_material_capture_streamer_accessors() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let RenderBackend { device, queue, .. } = backend;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-prepare-context-material-capture-test-encoder"),
        });
        let streamer = test_resource_streamer(&device, &queue);
        let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(4, 4));
        let mut external_buffer_bindings = Vec::new();
        let context = RuntimePrepareCollectorContext::new(
            &device,
            &queue,
            &mut encoder,
            &streamer,
            &frame,
            &mut external_buffer_bindings,
        );

        let missing_material = ResourceId::from_stable_label("res://materials/missing.zmat");
        assert!(context.material_capture_seed(&missing_material).is_none());
        assert!(context.sample_texture_rgba(None, [0.5, 0.5]).is_none());
    }

    fn test_resource_streamer(device: &wgpu::Device, queue: &wgpu::Queue) -> ResourceStreamer {
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-runtime-prepare-context-test-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        ResourceStreamer::new_for_test(
            Arc::new(crate::asset::pipeline::manager::ProjectAssetManager::default()),
            device,
            queue,
            &texture_layout,
        )
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
