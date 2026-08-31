//! Render hardware interface contracts and descriptors.

mod capabilities;
mod descriptors;
mod device;
mod device_fault;
mod device_profile;
mod diagnostic_query;
mod diagnostic_readback;
mod memory;
mod native_surface;
mod submission;
mod submission_packet;
mod surface;
mod texture_copy;
mod texture_view;
mod ui_surface;
mod upload;

pub use capabilities::{
    AccelerationStructureCaps, RenderAdapterInfo, RenderBackendCaps,
    RenderDebugInstrumentationStatus, RenderDeviceLimits, RenderOperation, RenderOperationMatrix,
    RenderOperationSupport, RenderQueueClass, UnsupportedRenderOperation,
};
pub use descriptors::{
    AddressMode, BindGroupLayoutDesc, BindGroupLayoutEntryDesc, BindingResourceType,
    BlendComponentDesc, BlendFactor, BlendOperation, BlendStateDesc, BufferDesc, BufferUsage,
    ColorTargetDesc, ColorWriteMask, CompareFunction, CullMode, DepthStencilStateDesc, FilterMode,
    FrontFace, MipmapFilterMode, PipelineDesc, PipelineKind, PipelineLayoutDesc, PresentMode,
    PrimitiveStateDesc, PrimitiveTopology, RasterPipelineStateDesc, SamplerBindingType,
    SamplerDesc, ShaderModuleDesc, ShaderStage, StorageTextureAccess, StorageTextureBindingDesc,
    SwapchainDesc, TextureDesc, TextureDimension, TextureFormat, TextureResidency,
    TextureSampleType, TextureUsage, TextureViewAspect, TextureViewDimension, VertexAttributeDesc,
    VertexBufferLayoutDesc, VertexFormat, VertexInputLayoutDesc, VertexStepMode,
};
pub use device::{
    BindGroupBufferBinding, BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource,
    BindGroupHandle, BindGroupLayoutHandle, BufferHandle, CommandList, CommandListCommand,
    IndexFormat, PipelineHandle, PipelineLayoutHandle, RenderClearColor, RenderDevice,
    RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp, RenderPassStoreOp,
    RenderPassTextureViewDesc, RenderResourceHandleAllocationError, RenderResourceHandleAllocator,
    RenderResourceHandleError, RenderResourceKind, RenderScissorRect, RenderViewportDesc, RhiError,
    SamplerHandle, ShaderModuleHandle, TextureHandle, TextureViewHandle, TransientAllocatorStats,
};
pub use device_fault::{DeviceAdmissionError, DeviceFaultGate, DeviceFaultKind, DeviceFaultRecord};
pub use device_profile::{
    AdapterSelectionError, AdapterSelectionPolicy, AdapterSelectionReceipt, DeviceGeneration,
    DeviceId, RejectedAdapter, RejectedAdapterReason, RenderAdapterCatalog, RenderAdapterClass,
    RenderAdapterFacts, RenderAdapterSelector, RenderBackendKind, RenderDeviceFeature,
    RenderDeviceFeatureNegotiation, RenderDeviceFeatureSet, RenderDeviceNegotiationError,
    RenderDeviceProfile, RenderDeviceQueueTopology, RenderDeviceRequestFailure,
    RenderDeviceRequestPolicy,
};
pub use diagnostic_query::{
    aggregate_diagnostic_query_results, DiagnosticPassQueryScope, DiagnosticPassResult,
    DiagnosticPipelineStatistics, DiagnosticQueryDecodeError, DiagnosticQueryPlan,
    DiagnosticQueryPlanError, PassDiagnosticId, PipelineStatisticsScope, TimestampScope,
    PIPELINE_STATISTIC_COUNTERS_PER_QUERY,
};
pub use diagnostic_readback::{
    DiagnosticFrameKey, DiagnosticReadbackAdmission, DiagnosticReadbackBudget,
    DiagnosticReadbackError, DiagnosticReadbackKind, DiagnosticReadbackReceipt,
    DiagnosticReadbackRequestId, DiagnosticReadbackTerminal, DiagnosticReadbackTracker,
};
pub use memory::{GpuMemoryBudget, GpuMemoryClass, GpuMemorySnapshot};
pub use native_surface::RenderNativeSurfaceTarget;
pub use submission::{
    SubmissionHistory, SubmissionLimits, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket,
};
pub use submission_packet::RhiSubmissionPacket;
pub use surface::{
    RenderSurfaceDescriptor, RenderSurfaceHandleAllocator, RenderSurfaceHandleError,
    SurfaceAcquireOutcome, SurfaceFrameId, SurfaceFrameLease, SurfaceFrameTerminal,
    SurfaceFrameTerminalHistory, SurfacePresentReceipt, SurfaceReconfigureReason,
    SurfaceRetryReason, SurfaceSession, SurfaceSessionCreateOutcome, SurfaceSessionReceipt,
};
pub use texture_copy::{TextureCopyAspect, TextureCopyRegion};
pub use texture_view::TextureViewDesc;
pub use ui_surface::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDescriptor, UiSurfaceDrawList,
    UiSurfaceImagePayload, UiSurfaceImageResource, UiSurfaceImageResourceTable,
    UiSurfaceImageUvRect, UiSurfacePresentOutcome, UiSurfacePresentStats,
    UiSurfacePresentStatsAccumulator, UiSurfacePresenter, UiSurfaceRect,
    UiSurfaceResolvedCommandKind, UiSurfaceStyle, UiSurfaceStyleHandle, UiSurfaceStyledPayload,
    UiSurfaceTextStyle,
};
pub use upload::{BufferUpload, BufferUploadBatch, TextureUpload, TextureUploadBatch};

#[cfg(test)]
mod tests;
