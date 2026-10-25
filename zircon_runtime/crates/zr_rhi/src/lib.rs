//! Render hardware interface contracts and descriptors.

#[path = "capabilities.rs"]
mod capabilities;
#[path = "descriptors.rs"]
mod descriptors;
#[path = "device.rs"]
mod device;
#[path = "device_fault.rs"]
mod device_fault;
#[path = "device_profile.rs"]
mod device_profile;
#[path = "diagnostic_query.rs"]
mod diagnostic_query;
#[path = "diagnostic_readback.rs"]
mod diagnostic_readback;
#[path = "memory.rs"]
mod memory;
#[path = "native_surface.rs"]
mod native_surface;
#[path = "submission.rs"]
mod submission;
#[path = "submission_packet.rs"]
mod submission_packet;
#[path = "surface.rs"]
mod surface;
#[path = "texture_copy/mod.rs"]
mod texture_copy;
#[path = "texture_view.rs"]
mod texture_view;
#[path = "ui_surface.rs"]
mod ui_surface;
#[path = "upload.rs"]
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
    RenderAdapterFacts, RenderAdapterSelector, RenderBackendKind, RenderDeviceBudgets,
    RenderDeviceFeature, RenderDeviceFeatureNegotiation, RenderDeviceFeatureSet,
    RenderDeviceNegotiationError, RenderDeviceProfile, RenderDeviceQueueTopology,
    RenderDeviceRequestFailure, RenderDeviceRequestPolicy,
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
pub use submission_packet::{
    RhiGraphAccessId, RhiGraphAccessRange, RhiGraphExecutionAccess, RhiGraphExecutionPass,
    RhiGraphExecutionReceipt, RhiGraphExecutionReceiptError, RhiGraphExecutionTransition,
    RhiGraphPhysicalResourceLease, RhiGraphQueueLane, RhiGraphResourceAccessKind,
    RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind, RhiGraphResourceState,
    RhiSubmissionPacket,
};
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
    UiSurfaceFontSnapshot, UiSurfaceFontSource, UiSurfaceImagePayload, UiSurfaceImageResource,
    UiSurfaceImageResourceTable, UiSurfaceImageUvRect, UiSurfacePresentOutcome,
    UiSurfacePresentStats, UiSurfacePresentStatsAccumulator, UiSurfacePresenter, UiSurfaceRect,
    UiSurfaceResolvedCommandKind, UiSurfaceStyle, UiSurfaceStyleHandle, UiSurfaceStyledPayload,
    UiSurfaceTextFace, UiSurfaceTextLayoutRun, UiSurfaceTextLayoutSnapshot, UiSurfaceTextLine,
    UiSurfaceTextRequest, UiSurfaceTextStyle,
};
pub use upload::{BufferUpload, BufferUploadBatch, TextureUpload, TextureUploadBatch};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub use ui_surface::{
    UiSurfaceFramePixelFormat, UiSurfaceFrameReadback, UiSurfaceFrameReadbackRequest,
    UiSurfaceImageAlphaMode, UiSurfaceImageReadback, UiSurfaceImageReadbackRequest,
};
