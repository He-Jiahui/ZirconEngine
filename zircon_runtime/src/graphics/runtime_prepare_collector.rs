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
use crate::graphics::{
    GraphicsError, RenderPassDeviceEpoch, RuntimePrepareDeviceEpoch,
    RuntimePrepareMeshGeometrySeed, ViewportRenderFrame,
};
use crate::rhi::{BufferDesc, DeviceGeneration, DeviceId, RenderDeviceProfile};
use zr_rhi_wgpu::{GpuPassTimer, GpuPassTimestampScope, WgpuBufferUpload, WgpuBufferUploadBatch};

const DEFAULT_RUNTIME_PREPARE_MAX_IN_FLIGHT_READBACK_FRAMES: usize = 3;

#[path = "runtime_prepare_collector/gpu_readback.rs"]
mod gpu_readback;
use gpu_readback::RuntimePrepareGpuReadbackRequest;

pub trait RuntimePrepareCollector: Send + Sync {
    fn requests_gpu_readback(&self) -> bool {
        false
    }

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
    device: &'a wgpu::Device,
    device_epoch: RuntimePrepareDeviceEpoch,
    encoder: &'a mut wgpu::CommandEncoder,
    frame_extract: &'a RenderFrameExtract,
    frame: &'a ViewportRenderFrame,
    streamer: &'a ResourceStreamer,
    external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
    gpu_readbacks: Option<&'a mut Vec<RuntimePrepareGpuReadbackRequest>>,
    gpu_work_admitted: bool,
    gpu_pass_timer: Option<&'a mut GpuPassTimer>,
    gpu_pass_profiles: Option<&'a mut Vec<RuntimePrepareGpuPassProfile>>,
    buffer_uploads: WgpuBufferUploadBatch,
    frame_transactions: Vec<RuntimePrepareFrameTransaction>,
}

/// Runtime-prepare capability for CPU writes that join the frame upload transaction.
pub struct RuntimePrepareBufferUploadRecorder<'a> {
    uploads: &'a mut WgpuBufferUploadBatch,
}

/// Queue-free GPU recording authority for one runtime-prepare producer.
pub struct RuntimePrepareGpuRecordingContext<'a> {
    pub device: &'a wgpu::Device,
    pub device_epoch: RuntimePrepareDeviceEpoch,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub buffer_uploads: RuntimePrepareBufferUploadRecorder<'a>,
    pub frame_transactions: RuntimePrepareFrameTransactionRecorder<'a>,
}

/// Recorder for CPU state that becomes authoritative with an accepted runtime-prepare frame.
pub struct RuntimePrepareFrameTransactionRecorder<'a> {
    transactions: &'a mut Vec<RuntimePrepareFrameTransaction>,
}

impl<'a> RuntimePrepareFrameTransactionRecorder<'a> {
    pub fn new(transactions: &'a mut Vec<RuntimePrepareFrameTransaction>) -> Self {
        Self { transactions }
    }

    pub fn register(&mut self, transaction: RuntimePrepareFrameTransaction) {
        self.transactions.push(transaction);
    }
}

impl RuntimePrepareBufferUploadRecorder<'_> {
    pub fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        self.uploads
            .push(WgpuBufferUpload::from_bytes(buffer.clone(), offset, bytes));
    }
}

impl crate::graphics::scene::RenderPassBufferUploadSink for RuntimePrepareBufferUploadRecorder<'_> {
    fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        RuntimePrepareBufferUploadRecorder::write_buffer(self, buffer, offset, bytes);
    }
}

/// State transition prepared by a runtime collector and committed with the accepted scene frame.
///
/// Dropping the transaction before commit invokes its rollback action. Both actions must be
/// bounded, infallible render-thread bookkeeping and must not submit native GPU work.
pub struct RuntimePrepareFrameTransaction {
    label: &'static str,
    action: Option<Box<dyn RuntimePrepareFrameTransactionAction>>,
}

trait RuntimePrepareFrameTransactionAction: Send {
    fn commit(self: Box<Self>);
    fn rollback(self: Box<Self>);
}

struct ClosureRuntimePrepareFrameTransactionAction<Commit, Rollback> {
    commit: Commit,
    rollback: Rollback,
}

impl<Commit, Rollback> RuntimePrepareFrameTransactionAction
    for ClosureRuntimePrepareFrameTransactionAction<Commit, Rollback>
where
    Commit: FnOnce() + Send + 'static,
    Rollback: FnOnce() + Send + 'static,
{
    fn commit(self: Box<Self>) {
        let Self { commit, .. } = *self;
        commit();
    }

    fn rollback(self: Box<Self>) {
        let Self { rollback, .. } = *self;
        rollback();
    }
}

impl RuntimePrepareFrameTransaction {
    pub fn new(
        label: &'static str,
        commit: impl FnOnce() + Send + 'static,
        rollback: impl FnOnce() + Send + 'static,
    ) -> Self {
        Self {
            label,
            action: Some(Box::new(ClosureRuntimePrepareFrameTransactionAction {
                commit,
                rollback,
            })),
        }
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub(crate) fn commit(mut self) {
        if let Some(action) = self.action.take() {
            action.commit();
        }
    }
}

impl fmt::Debug for RuntimePrepareFrameTransaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimePrepareFrameTransaction")
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

impl Drop for RuntimePrepareFrameTransaction {
    fn drop(&mut self) {
        if let Some(action) = self.action.take() {
            action.rollback();
        }
    }
}

#[derive(Default)]
pub(crate) struct RuntimePrepareFramePacket {
    buffer_uploads: WgpuBufferUploadBatch,
    frame_transactions: Vec<RuntimePrepareFrameTransaction>,
}

impl RuntimePrepareFramePacket {
    pub(crate) fn append(&mut self, other: &mut Self) {
        self.buffer_uploads.append(&mut other.buffer_uploads);
        self.frame_transactions
            .append(&mut other.frame_transactions);
    }

    pub(crate) fn take_buffer_uploads(&mut self) -> WgpuBufferUploadBatch {
        std::mem::take(&mut self.buffer_uploads)
    }

    pub(crate) fn commit_frame_transactions(&mut self) {
        for transaction in self.frame_transactions.drain(..) {
            transaction.commit();
        }
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.buffer_uploads.is_empty() && self.frame_transactions.is_empty()
    }
}

impl<'a> RuntimePrepareCollectorContext<'a> {
    pub const MAX_IN_FLIGHT_GPU_READBACK_FRAMES: usize =
        DEFAULT_RUNTIME_PREPARE_MAX_IN_FLIGHT_READBACK_FRAMES;

    pub(crate) fn new(
        device: &'a wgpu::Device,
        device_epoch: RuntimePrepareDeviceEpoch,
        encoder: &'a mut wgpu::CommandEncoder,
        streamer: &'a ResourceStreamer,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Self {
        Self {
            device,
            device_epoch,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: None,
            gpu_work_admitted: false,
            gpu_pass_timer: None,
            gpu_pass_profiles: None,
            buffer_uploads: WgpuBufferUploadBatch::new(),
            frame_transactions: Vec::new(),
        }
    }

    pub(crate) fn new_with_gpu_readbacks(
        device: &'a wgpu::Device,
        device_epoch: RuntimePrepareDeviceEpoch,
        encoder: &'a mut wgpu::CommandEncoder,
        streamer: &'a ResourceStreamer,
        frame: &'a ViewportRenderFrame,
        external_buffer_bindings: &'a mut Vec<RuntimePrepareExternalBufferBinding>,
        gpu_readbacks: &'a mut Vec<RuntimePrepareGpuReadbackRequest>,
    ) -> Self {
        Self {
            device,
            device_epoch,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: Some(gpu_readbacks),
            gpu_work_admitted: true,
            gpu_pass_timer: None,
            gpu_pass_profiles: None,
            buffer_uploads: WgpuBufferUploadBatch::new(),
            frame_transactions: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_gpu_readbacks_and_gpu_work_admission(
        device: &'a wgpu::Device,
        device_epoch: RuntimePrepareDeviceEpoch,
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
            device_epoch,
            encoder,
            frame_extract: &frame.extract,
            frame,
            streamer,
            external_buffer_bindings,
            gpu_readbacks: gpu_work_admitted.then_some(gpu_readbacks),
            gpu_work_admitted,
            gpu_pass_timer,
            gpu_pass_profiles: Some(gpu_pass_profiles),
            buffer_uploads: WgpuBufferUploadBatch::new(),
            frame_transactions: Vec::new(),
        }
    }

    /// Records a CPU buffer write for the runtime-prepare transaction.
    ///
    /// The write is submitted only if the complete frame transaction succeeds. Runtime-prepare
    /// producers therefore cannot publish data ahead of graph validation or retain queue access.
    pub fn buffer_upload_recorder(&mut self) -> RuntimePrepareBufferUploadRecorder<'_> {
        RuntimePrepareBufferUploadRecorder {
            uploads: &mut self.buffer_uploads,
        }
    }

    pub fn gpu_recording_context(&mut self) -> RuntimePrepareGpuRecordingContext<'_> {
        RuntimePrepareGpuRecordingContext {
            device: self.device,
            device_epoch: self.device_epoch,
            encoder: &mut *self.encoder,
            buffer_uploads: RuntimePrepareBufferUploadRecorder {
                uploads: &mut self.buffer_uploads,
            },
            frame_transactions: RuntimePrepareFrameTransactionRecorder::new(
                &mut self.frame_transactions,
            ),
        }
    }

    /// Attaches CPU-side prepared state to the same acceptance boundary as this frame's uploads.
    pub fn register_frame_transaction(&mut self, transaction: RuntimePrepareFrameTransaction) {
        self.frame_transactions.push(transaction);
    }

    pub(crate) fn take_frame_packet(&mut self) -> RuntimePrepareFramePacket {
        RuntimePrepareFramePacket {
            buffer_uploads: std::mem::take(&mut self.buffer_uploads),
            frame_transactions: std::mem::take(&mut self.frame_transactions),
        }
    }

    /// New runtime-prepare GPU work is valid only when its shared completion frame exists.
    pub fn gpu_work_admitted(&self) -> bool {
        self.gpu_work_admitted
    }

    /// Returns the immutable device generation associated with this dispatch.
    pub fn device_epoch(&self) -> RuntimePrepareDeviceEpoch {
        self.device_epoch
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

    pub fn frame_extract(&self) -> &'a RenderFrameExtract {
        self.frame_extract
    }

    /// Returns meshes from the canonical runtime extract rather than the compatibility snapshot.
    pub fn scene_meshes(&self) -> &'a [RenderMeshSnapshot] {
        &self.frame_extract.geometry.meshes
    }

    pub fn scene_snapshot(&self) -> &'a RenderSceneSnapshot {
        &self.frame.scene
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }

    pub fn prepared_runtime_sidebands(&self) -> &'a RenderPreparedRuntimeSidebands {
        self.frame.prepared_runtime_sidebands()
    }

    pub fn prepared_plugin_renderer_outputs(&self) -> &'a RenderPluginRendererOutputs {
        &self.prepared_runtime_sidebands().plugin_renderer_outputs
    }

    pub fn prepared_hybrid_gi_readback_outputs(&self) -> &'a RenderHybridGiReadbackOutputs {
        self.prepared_runtime_sidebands()
            .hybrid_gi_readback_outputs()
    }

    pub fn prepared_virtual_geometry_readback_outputs(
        &self,
    ) -> &'a RenderVirtualGeometryReadbackOutputs {
        self.prepared_runtime_sidebands()
            .virtual_geometry_readback_outputs()
    }

    pub fn prepared_hybrid_gi_evictable_probe_ids(&self) -> &'a [u32] {
        self.prepared_runtime_sidebands()
            .hybrid_gi_evictable_probe_ids()
    }

    pub fn prepared_virtual_geometry_evictable_page_ids(&self) -> &'a [u32] {
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

    /// Registers a borrowed external buffer together with the physical
    /// descriptor that schema-backed graph declarations validate before use.
    pub fn register_external_buffer_binding_with_physical_desc(
        &mut self,
        logical_name: impl Into<String>,
        buffer: &wgpu::Buffer,
        physical_desc: BufferDesc,
    ) {
        let logical_name = logical_name.into();
        let backing_name = format!("{logical_name}:runtime-prepare");
        self.register_external_buffer_binding_with_backing_and_physical_desc(
            logical_name,
            backing_name,
            buffer,
            physical_desc,
        );
    }

    /// Registers a borrowed external buffer with a stable backing identity
    /// and producer-supplied descriptor for graph materialization validation.
    pub fn register_external_buffer_binding_with_backing_and_physical_desc(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
        physical_desc: BufferDesc,
    ) {
        self.external_buffer_bindings.push(
            RuntimePrepareExternalBufferBinding::new_with_physical_desc(
                logical_name,
                backing_name,
                buffer,
                physical_desc,
            ),
        );
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

    /// Static-name variant of physical descriptor registration. It preserves
    /// the allocation-free logical/backing identity path for first-party
    /// schema-backed external buffers.
    pub fn register_static_external_buffer_binding_with_backing_and_physical_desc(
        &mut self,
        logical_name: &'static str,
        backing_name: &'static str,
        buffer: &wgpu::Buffer,
        physical_desc: BufferDesc,
    ) {
        self.external_buffer_bindings.push(
            RuntimePrepareExternalBufferBinding::new_static_with_physical_desc(
                logical_name,
                backing_name,
                buffer,
                physical_desc,
            ),
        );
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
        requests.push(RuntimePrepareGpuReadbackRequest::new(
            name.into(),
            buffer,
            range,
            self.device_epoch,
            Arc::clone(&readback.completion),
        ));
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

#[derive(Clone, Debug)]
pub struct RuntimePrepareMaterialCaptureSeed {
    pub base_color: Vec4,
    pub emissive: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub occlusion_strength: f32,
    pub normal_scale: f32,
    pub double_sided: bool,
    pub alpha_blend: bool,
    pub alpha_cutoff: Option<f32>,
    pub cast_shadows: bool,
    pub base_color_texture: Option<ResourceId>,
    pub base_color_texture_revision: Option<u64>,
    pub base_color_texture_center_rgba: Option<Vec4>,
    pub normal_texture: Option<ResourceId>,
    pub normal_texture_revision: Option<u64>,
    pub normal_texture_center_rgba: Option<Vec4>,
    pub metallic_roughness_texture: Option<ResourceId>,
    pub metallic_roughness_texture_revision: Option<u64>,
    pub metallic_roughness_texture_center_rgba: Option<Vec4>,
    pub occlusion_texture: Option<ResourceId>,
    pub occlusion_texture_revision: Option<u64>,
    pub occlusion_texture_center_rgba: Option<Vec4>,
    pub emissive_texture: Option<ResourceId>,
    pub emissive_texture_revision: Option<u64>,
    pub emissive_texture_center_rgba: Option<Vec4>,
}

impl RuntimePrepareMaterialCaptureSeed {
    fn from_material_capture_seed(seed: MaterialCaptureSeed) -> Self {
        Self {
            base_color: seed.base_color,
            emissive: seed.emissive,
            metallic: seed.metallic,
            roughness: seed.roughness,
            occlusion_strength: seed.occlusion_strength,
            normal_scale: seed.normal_scale,
            double_sided: seed.double_sided,
            alpha_blend: seed.alpha_blend,
            alpha_cutoff: seed.alpha_cutoff,
            cast_shadows: seed.cast_shadows,
            base_color_texture: seed.base_color_texture,
            base_color_texture_revision: seed.base_color_texture_revision,
            base_color_texture_center_rgba: seed.base_color_texture_center_rgba,
            normal_texture: seed.normal_texture,
            normal_texture_revision: seed.normal_texture_revision,
            normal_texture_center_rgba: seed.normal_texture_center_rgba,
            metallic_roughness_texture: seed.metallic_roughness_texture,
            metallic_roughness_texture_revision: seed.metallic_roughness_texture_revision,
            metallic_roughness_texture_center_rgba: seed.metallic_roughness_texture_center_rgba,
            occlusion_texture: seed.occlusion_texture,
            occlusion_texture_revision: seed.occlusion_texture_revision,
            occlusion_texture_center_rgba: seed.occlusion_texture_center_rgba,
            emissive_texture: seed.emissive_texture,
            emissive_texture_revision: seed.emissive_texture_revision,
            emissive_texture_center_rgba: seed.emissive_texture_center_rgba,
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimePrepareExternalBufferBinding {
    logical_name: Cow<'static, str>,
    backing_name: Cow<'static, str>,
    buffer: wgpu::Buffer,
    physical_desc: Option<BufferDesc>,
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
            physical_desc: None,
        }
    }

    pub(crate) fn new_with_physical_desc(
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
        physical_desc: BufferDesc,
    ) -> Self {
        Self {
            logical_name: Cow::Owned(logical_name.into()),
            backing_name: Cow::Owned(backing_name.into()),
            buffer: buffer.clone(),
            physical_desc: Some(physical_desc),
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
            physical_desc: None,
        }
    }

    fn new_static_with_physical_desc(
        logical_name: &'static str,
        backing_name: &'static str,
        buffer: &wgpu::Buffer,
        physical_desc: BufferDesc,
    ) -> Self {
        Self {
            logical_name: Cow::Borrowed(logical_name),
            backing_name: Cow::Borrowed(backing_name),
            buffer: buffer.clone(),
            physical_desc: Some(physical_desc),
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

    pub(crate) fn physical_desc(&self) -> Option<&BufferDesc> {
        self.physical_desc.as_ref()
    }
}

/// One frame-local external-buffer handoff qualified by the device generation
/// that admitted its runtime-prepare collectors.
pub(crate) struct RuntimePrepareExternalBufferBindingPacket {
    device_id: DeviceId,
    device_generation: DeviceGeneration,
    bindings: Vec<RuntimePrepareExternalBufferBinding>,
}

impl RuntimePrepareExternalBufferBindingPacket {
    pub(crate) fn new(
        device_profile: &RenderDeviceProfile,
        bindings: Vec<RuntimePrepareExternalBufferBinding>,
    ) -> Option<Self> {
        (!bindings.is_empty()).then(|| Self {
            device_id: device_profile.device_id(),
            device_generation: device_profile.generation(),
            bindings,
        })
    }

    pub(crate) fn bindings(&self) -> &[RuntimePrepareExternalBufferBinding] {
        &self.bindings
    }

    pub(crate) fn ensure_device_epoch(
        &self,
        expected: Option<RenderPassDeviceEpoch>,
    ) -> Result<(), String> {
        let Some(expected) = expected else {
            return Err(
                "runtime prepare external buffer packet reached graph binding before the execution device epoch was established"
                    .to_owned(),
            );
        };
        let (expected_device_id, expected_generation) = expected.raw_parts();
        let actual_device_id = self.device_id.raw();
        let actual_generation = self.device_generation.raw();
        if expected_device_id != actual_device_id || expected_generation != actual_generation {
            return Err(format!(
                "runtime prepare external buffer packet belongs to device {actual_device_id} generation {actual_generation}, expected device {expected_device_id} generation {expected_generation}"
            ));
        }
        Ok(())
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

    pub fn requests_gpu_readback(&self) -> bool {
        self.collector.requests_gpu_readback()
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
