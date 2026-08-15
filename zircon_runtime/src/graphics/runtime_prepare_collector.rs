use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameExtract, RenderHybridGiReadbackOutputs, RenderMeshSnapshot,
    RenderPluginRendererOutputs, RenderPreparedRuntimeSidebands, RenderSceneSnapshot,
    RenderVirtualGeometryReadbackOutputs,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::MaterialCaptureSeed;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::{GraphicsError, RuntimePrepareMeshGeometrySeed, ViewportRenderFrame};
use zr_rhi_wgpu::{GpuPassTimer, GpuPassTimestampScope, GpuReadbackQueue, ReadbackError};

pub trait RuntimePrepareCollector: Send + Sync {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError>;
}

/// Opaque shared-timer scope for one runtime-prepare GPU dispatch group.
pub struct RuntimePrepareGpuPassScope {
    pass_name: String,
    timestamp_scope: Option<GpuPassTimestampScope>,
}

pub(crate) struct RuntimePrepareGpuPassProfile {
    pub(crate) pass_name: String,
    pub(crate) executor_id: String,
    pub(crate) budget_key: RenderBudgetKey,
    pub(crate) cpu_elapsed_micros: u64,
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
    gpu_work_admitted: bool,
    gpu_pass_timer: Option<&'a mut GpuPassTimer>,
    gpu_pass_profiles: Option<&'a mut Vec<RuntimePrepareGpuPassProfile>>,
}

impl<'a> RuntimePrepareCollectorContext<'a> {
    pub const MAX_IN_FLIGHT_GPU_READBACK_FRAMES: usize = GpuReadbackQueue::FRAME_SLOTS;

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
            gpu_work_admitted: false,
            gpu_pass_timer: None,
            gpu_pass_profiles: None,
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
            gpu_work_admitted: true,
            gpu_pass_timer: None,
            gpu_pass_profiles: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_gpu_readbacks_and_gpu_work_admission(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        streamer: &'a ResourceStreamer,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
        gpu_readbacks: &'a mut Vec<RuntimePrepareGpuReadbackRequest>,
        gpu_work_admitted: bool,
        gpu_pass_timer: Option<&'a mut GpuPassTimer>,
        gpu_pass_profiles: &'a mut Vec<RuntimePrepareGpuPassProfile>,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: gpu_work_admitted.then_some(gpu_readbacks),
            gpu_work_admitted,
            gpu_pass_timer,
            gpu_pass_profiles: Some(gpu_pass_profiles),
        }
    }

    /// New runtime-prepare GPU work is valid only when its shared completion frame exists.
    pub fn gpu_work_admitted(&self) -> bool {
        self.gpu_work_admitted
    }

    pub fn begin_gpu_pass(&mut self, pass_name: impl Into<String>) -> RuntimePrepareGpuPassScope {
        let pass_name = pass_name.into();
        let timestamp_scope = if self.gpu_work_admitted {
            let (gpu_pass_timer, encoder) = (&mut self.gpu_pass_timer, &mut self.encoder);
            gpu_pass_timer
                .as_deref_mut()
                .and_then(|timer| timer.begin_pass(encoder, &pass_name))
        } else {
            None
        };
        RuntimePrepareGpuPassScope {
            pass_name,
            timestamp_scope,
        }
    }

    pub fn end_gpu_pass(
        &mut self,
        scope: RuntimePrepareGpuPassScope,
        executor_id: impl Into<String>,
        budget_key: RenderBudgetKey,
        cpu_elapsed_micros: u64,
    ) {
        if !self.gpu_work_admitted {
            return;
        }
        let RuntimePrepareGpuPassScope {
            pass_name,
            timestamp_scope,
        } = scope;
        self.close_gpu_pass_timestamp_scope(timestamp_scope);
        if let Some(profiles) = self.gpu_pass_profiles.as_deref_mut() {
            profiles.push(RuntimePrepareGpuPassProfile {
                pass_name,
                executor_id: executor_id.into(),
                budget_key,
                cpu_elapsed_micros,
            });
        }
    }

    /// Closes an admitted scope when its attempted work encoded no GPU dispatch.
    pub fn discard_gpu_pass(&mut self, scope: RuntimePrepareGpuPassScope) {
        if self.gpu_work_admitted {
            self.close_gpu_pass_timestamp_scope(scope.timestamp_scope);
        }
    }

    fn close_gpu_pass_timestamp_scope(&mut self, timestamp_scope: Option<GpuPassTimestampScope>) {
        if let Some(timestamp_scope) = timestamp_scope {
            let (gpu_pass_timer, encoder) = (&mut self.gpu_pass_timer, &mut self.encoder);
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.end_pass(encoder, timestamp_scope);
            }
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

    pub fn mesh_geometry_seed(
        &self,
        mesh: &RenderMeshSnapshot,
    ) -> Option<RuntimePrepareMeshGeometrySeed> {
        self.streamer.render_mesh_geometry_seed(mesh)
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

    /// Registers compile-time resource identities without allocating binding names per frame.
    pub fn register_static_external_buffer_binding_with_backing(
        &mut self,
        logical_name: &'static str,
        backing_name: &'static str,
        buffer: &wgpu::Buffer,
    ) {
        self.external_buffer_bindings
            .push(RuntimePrepareExternalBufferBinding::new_static(
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
    pub occlusion_strength: f32,
    pub double_sided: bool,
    pub alpha_blend: bool,
    pub alpha_cutoff: Option<f32>,
    pub cast_shadows: bool,
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
            occlusion_strength: seed.occlusion_strength,
            double_sided: seed.double_sided,
            alpha_blend: seed.alpha_blend,
            alpha_cutoff: seed.alpha_cutoff,
            cast_shadows: seed.cast_shadows,
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
    logical_name: Cow<'static, str>,
    backing_name: Cow<'static, str>,
    buffer: wgpu::Buffer,
}

impl RuntimePrepareExternalBufferBinding {
    pub(crate) fn new(
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) -> Self {
        Self {
            logical_name: Cow::Owned(logical_name.into()),
            backing_name: Cow::Owned(backing_name.into()),
            buffer: buffer.clone(),
        }
    }

    fn new_static(
        logical_name: &'static str,
        backing_name: &'static str,
        buffer: &wgpu::Buffer,
    ) -> Self {
        Self {
            logical_name: Cow::Borrowed(logical_name),
            backing_name: Cow::Borrowed(backing_name),
            buffer: buffer.clone(),
        }
    }

    pub(crate) fn logical_name(&self) -> &str {
        self.logical_name.as_ref()
    }

    pub(crate) fn backing_name(&self) -> &str {
        self.backing_name.as_ref()
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
#[path = "runtime_prepare_collector/tests.rs"]
mod tests;
