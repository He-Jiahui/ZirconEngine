use std::time::Duration;

use thiserror::Error;

use crate::capabilities::{
    RenderDeviceLimits, RenderOperation, RenderOperationSupport, RenderQueueClass,
    UnsupportedRenderOperation,
};
use crate::descriptors::{BufferUsage, PipelineKind, TextureUsage};
use crate::device_fault::DeviceAdmissionError;
use crate::device_profile::{
    DeviceGeneration, DeviceId, RenderAdapterFacts, RenderDeviceFeatureSet,
    RenderDeviceQueueTopology,
};
use crate::diagnostic_query::DiagnosticQueryPlanError;
use crate::diagnostic_readback::DiagnosticReadbackError;
use crate::memory::GpuMemoryClass;
use crate::submission::{SubmissionStatus, SubmissionTicket};
use crate::surface::{RenderSurfaceHandleError, SurfaceFrameId, SurfaceFrameTerminal};

use super::handles::{RenderResourceHandleAllocationError, RenderResourceHandleError};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RhiError {
    #[error("render queue `{0:?}` is not supported by this backend")]
    UnsupportedQueue(RenderQueueClass),
    #[error("render operation `{operation:?}` admission failed with support `{support:?}`")]
    UnsupportedOperation {
        operation: RenderOperation,
        support: RenderOperationSupport,
    },
    #[error("device admission failed: {0:?}")]
    DeviceAdmission(DeviceAdmissionError),
    #[error(
        "native device context adapter {context_adapter:?} does not match profile adapter {profile_adapter:?}"
    )]
    NativeContextAdapterMismatch {
        profile_adapter: RenderAdapterFacts,
        context_adapter: RenderAdapterFacts,
    },
    #[error(
        "native device context limits {context_limits:?} do not match profile limits {profile_limits:?}"
    )]
    NativeContextDeviceLimitsMismatch {
        profile_limits: RenderDeviceLimits,
        context_limits: RenderDeviceLimits,
    },
    #[error(
        "native device features {context_features:?} do not match profile requested features {profile_features:?}"
    )]
    NativeContextRequestedFeaturesMismatch {
        profile_features: RenderDeviceFeatureSet,
        context_features: RenderDeviceFeatureSet,
    },
    #[error(
        "native WGPU context queue topology {context_topology:?} does not match profile topology {profile_topology:?}"
    )]
    NativeContextQueueTopologyMismatch {
        profile_topology: RenderDeviceQueueTopology,
        context_topology: RenderDeviceQueueTopology,
    },
    #[error("resource handle allocation failed: {0}")]
    ResourceHandleAllocation(#[from] RenderResourceHandleAllocationError),
    #[error("resource handle validation failed: {0}")]
    ResourceHandle(#[from] RenderResourceHandleError),
    #[error("buffer `{0}` does not exist")]
    UnknownBuffer(u64),
    #[error("texture `{0}` does not exist")]
    UnknownTexture(u64),
    #[error("texture view `{0}` does not exist")]
    UnknownTextureView(u64),
    #[error("sampler `{0}` does not exist")]
    UnknownSampler(u64),
    #[error("bind group layout `{0}` does not exist")]
    UnknownBindGroupLayout(u64),
    #[error("bind group `{0}` does not exist")]
    UnknownBindGroup(u64),
    #[error("shader module `{0}` does not exist")]
    UnknownShaderModule(u64),
    #[error("pipeline layout `{0}` does not exist")]
    UnknownPipelineLayout(u64),
    #[error("pipeline `{0}` does not exist")]
    UnknownPipeline(u64),
    #[error("render surface is unavailable: {0}")]
    SurfaceUnavailable(String),
    #[error("invalid surface descriptor `{label:?}`: {reason}")]
    InvalidSurfaceDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error(transparent)]
    SurfaceHandle(#[from] RenderSurfaceHandleError),
    #[error("surface frame `{frame:?}` was already terminalized as `{terminal:?}")]
    SurfaceFrameAlreadyTerminal {
        frame: SurfaceFrameId,
        terminal: SurfaceFrameTerminal,
    },
    #[error(
        "surface frame `{frame:?}` cannot use submission `{submission:?}` from another device generation"
    )]
    SurfaceFrameSubmissionMismatch {
        frame: SurfaceFrameId,
        submission: SubmissionTicket,
    },
    #[error(
        "surface frame `{frame:?}` requires a submitted or completed ticket, received `{status:?}`"
    )]
    SurfaceFrameSubmissionNotReady {
        frame: SurfaceFrameId,
        status: SubmissionStatus,
    },
    #[error("surface frame `{frame:?}` was not referenced by submitted ticket `{submission:?}`")]
    SurfaceFrameSubmissionMissingTarget {
        frame: SurfaceFrameId,
        submission: SubmissionTicket,
    },
    #[error("surface frame lease `{frame:?}` does not match the active session target")]
    SurfaceFrameLeaseMismatch { frame: SurfaceFrameId },
    #[error("surface frame `{frame:?}` cleanup failed: {cleanup}; original failure: {source}")]
    SurfaceFrameCleanupFailed {
        frame: SurfaceFrameId,
        cleanup: Box<RhiError>,
        source: Box<RhiError>,
    },
    #[error("surface-owned texture `{texture}` must be released by present or discard")]
    SurfaceOwnedTexture { texture: u64 },
    #[error("surface-owned texture view `{view}` must be released by present or discard")]
    SurfaceOwnedTextureView { view: u64 },
    #[error("submission ticket `{0:?}` was not issued by this device generation")]
    UnknownSubmissionTicket(SubmissionTicket),
    #[error("submission packet must contain at least one command list")]
    EmptySubmissionPacket,
    #[error("render upload batch must contain at least one write")]
    EmptyUploadBatch,
    #[error("render upload batch payload byte count overflows u64")]
    UploadByteCountOverflow,
    #[error("render upload source range {start}..{end} exceeds payload length {payload_bytes}")]
    InvalidUploadSourceRange {
        start: usize,
        end: usize,
        payload_bytes: usize,
    },
    #[error("a command pass diagnostic scope requires a submission query plan")]
    DiagnosticQueryPlanRequired,
    #[error("a diagnostic query plan attached to a submission packet needs a frame index")]
    DiagnosticQueryFrameIndexRequired,
    #[error("a diagnostic pass scope must reserve at least one query kind")]
    EmptyDiagnosticPassScope,
    #[error("submission packet diagnostic query plan is invalid: {0}")]
    DiagnosticQueryPlan(#[from] DiagnosticQueryPlanError),
    #[error(
        "submission packet queue `{packet_queue:?}` does not match command-list queue `{command_queue:?}`"
    )]
    SubmissionPacketQueueMismatch {
        packet_queue: RenderQueueClass,
        command_queue: RenderQueueClass,
    },
    #[error(
        "submission packet belongs to device `{packet_device_id:?}` generation `{packet_generation:?}`, not device `{device_id:?}` generation `{generation:?}`"
    )]
    SubmissionPacketDeviceMismatch {
        packet_device_id: DeviceId,
        packet_generation: DeviceGeneration,
        device_id: DeviceId,
        generation: DeviceGeneration,
    },
    #[error(
        "submission sequence space is exhausted for device `{device_id:?}` generation `{generation:?}"
    )]
    SubmissionSequenceExhausted {
        device_id: DeviceId,
        generation: DeviceGeneration,
    },
    #[error(
        "submission poll sequence space is exhausted for device `{device_id:?}` generation `{generation:?}`"
    )]
    SubmissionPollSequenceExhausted {
        device_id: DeviceId,
        generation: DeviceGeneration,
    },
    #[error("submission ticket `{ticket:?}` did not reach a terminal state within {timeout:?}")]
    SubmissionWaitTimedOut {
        ticket: SubmissionTicket,
        timeout: Duration,
    },
    #[error("submission ticket `{ticket:?}` cannot be cancelled from state `{status:?}`")]
    SubmissionCannotCancel {
        ticket: SubmissionTicket,
        status: SubmissionStatus,
    },
    #[error("submission ticket `{ticket:?}` cannot accept a packet from state `{status:?}`")]
    SubmissionNotAcceptingPacket {
        ticket: SubmissionTicket,
        status: SubmissionStatus,
    },
    #[error("native device completion poll failed: {reason}")]
    NativeDevicePoll { reason: String },
    #[error(
        "{class:?} memory admission exceeds budget: current {current_bytes} bytes, request {requested_bytes} bytes, limit {limit_bytes} bytes"
    )]
    MemoryBudgetExceeded {
        class: GpuMemoryClass,
        current_bytes: u64,
        requested_bytes: u64,
        limit_bytes: u64,
    },
    #[error("pending upload count {pending_uploads} exceeds configured limit {limit}")]
    UploadBackpressure {
        pending_uploads: usize,
        limit: usize,
    },
    #[error(
        "unresolved submission count {unresolved_submissions} exceeds configured limit {limit}"
    )]
    SubmissionBackpressure {
        unresolved_submissions: usize,
        limit: usize,
    },
    #[error("{class:?} allocation could not reserve {requested_bytes} bytes")]
    ResourceAllocationFailed {
        class: GpuMemoryClass,
        requested_bytes: u64,
    },
    #[error("invalid buffer descriptor `{label:?}`: {reason}")]
    InvalidBufferDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid texture descriptor `{label:?}`: {reason}")]
    InvalidTextureDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid texture view descriptor `{label:?}`: {reason}")]
    InvalidTextureViewDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error(
        "texture `{texture}` cannot be destroyed while {live_views} texture views remain live"
    )]
    TextureHasLiveViews { texture: u64, live_views: u32 },
    #[error("invalid sampler descriptor `{label:?}`: {reason}")]
    InvalidSamplerDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group layout descriptor `{label:?}`: {reason}")]
    InvalidBindGroupLayoutDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group descriptor `{label:?}`: {reason}")]
    InvalidBindGroupDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group usage: {reason}")]
    InvalidBindGroupUsage { reason: String },
    #[error("invalid render pass: {reason}")]
    InvalidRenderPass { reason: String },
    #[error("invalid compute pass: {reason}")]
    InvalidComputePass { reason: String },
    #[error("invalid shader module descriptor `{label:?}`: {reason}")]
    InvalidShaderModuleDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid pipeline layout descriptor `{label:?}`: {reason}")]
    InvalidPipelineLayoutDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid pipeline descriptor `{label:?}`: {reason}")]
    InvalidPipelineDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("readback range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    ReadbackOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error("readback is unavailable: {reason}")]
    ReadbackUnavailable { reason: String },
    #[error("diagnostic readback lifecycle failed: {0}")]
    DiagnosticReadback(#[from] DiagnosticReadbackError),
    #[error("write range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    WriteOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error(
        "texture write is outside texture `{texture}`: source bytes {source_bytes}, bytes per row {bytes_per_row}, mip {mip_level}, origin ({origin_x}, {origin_y}, {origin_z}), width {width}, height {height}, depth or array layers {depth_or_array_layers}"
    )]
    TextureWriteOutOfRange {
        texture: u64,
        source_bytes: u64,
        bytes_per_row: u64,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
    },
    #[error("buffer `{buffer}` usage {actual:?} does not include required usage {required:?}")]
    InvalidBufferUsage {
        buffer: u64,
        required: BufferUsage,
        actual: BufferUsage,
    },
    #[error("texture `{texture}` usage {actual:?} does not include required usage {required:?}")]
    InvalidTextureUsage {
        texture: u64,
        required: TextureUsage,
        actual: TextureUsage,
    },
    #[error("pipeline `{pipeline}` kind {actual:?} does not satisfy required kind {required:?}")]
    InvalidPipelineUsage {
        pipeline: u64,
        required: PipelineKind,
        actual: PipelineKind,
    },
    #[error("command `{command}` cannot be recorded for queue `{queue:?}`")]
    InvalidCommandQueue {
        queue: RenderQueueClass,
        command: String,
    },
    #[error("invalid compute dispatch: {reason}")]
    InvalidComputeDispatch { reason: String },
    #[error("invalid raster draw: {reason}")]
    InvalidRasterDraw { reason: String },
    #[error("invalid debug marker: {reason}")]
    InvalidDebugMarker { reason: String },
    #[error("buffer binding range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    BufferBindingOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error(
        "buffer copy range is outside source `{source_buffer}` or destination `{destination_buffer}`: source offset {source_offset}, destination offset {destination_offset}, size {size}"
    )]
    BufferCopyOutOfRange {
        source_buffer: u64,
        destination_buffer: u64,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    },
    #[error("invalid copy command: {reason}")]
    InvalidCopy { reason: String },
    #[error(
        "buffer-to-texture copy is outside source `{source_buffer}` or destination `{destination_texture}`: source offset {source_offset}, bytes per row {bytes_per_row}, mip {mip_level}, origin ({origin_x}, {origin_y}, {origin_z}), width {width}, height {height}, depth or array layers {depth_or_array_layers}"
    )]
    BufferToTextureCopyOutOfRange {
        source_buffer: u64,
        destination_texture: u64,
        source_offset: u64,
        bytes_per_row: u64,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
    },
    #[error(
        "texture-to-buffer copy is outside source `{source_texture}` or destination `{destination_buffer}`: destination offset {destination_offset}, bytes per row {bytes_per_row}, mip {mip_level}, origin ({origin_x}, {origin_y}, {origin_z}), width {width}, height {height}, depth or array layers {depth_or_array_layers}"
    )]
    TextureToBufferCopyOutOfRange {
        source_texture: u64,
        destination_buffer: u64,
        destination_offset: u64,
        bytes_per_row: u64,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
    },
}

impl From<UnsupportedRenderOperation> for RhiError {
    fn from(error: UnsupportedRenderOperation) -> Self {
        Self::UnsupportedOperation {
            operation: error.operation,
            support: error.support,
        }
    }
}
