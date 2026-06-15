use crate::core::math::UVec2;

use super::{
    AdvancedProviderAvailability, AdvancedProviderReport, AntiAliasFallbackReport,
    RenderCameraTargetKind, RenderFrameExtract, RenderPostProcessEffectStackReport,
    RenderShadowExecutionReport, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source, SolariRuntimeReport, SolariSettings, TaaQualityPreset,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderViewportHandle(u64);

impl RenderViewportHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineHandle(u64);

impl RenderPipelineHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FrameHistoryHandle(u64);

impl FrameHistoryHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameHistoryInvalidationReason {
    NoPreviousFrame,
    ViewportResized,
    RenderSizeChanged,
    PipelineChanged,
    HistoryBindingChanged,
    FrameInputsChanged,
}

impl FrameHistoryInvalidationReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoPreviousFrame => "no_previous_frame",
            Self::ViewportResized => "viewport_resized",
            Self::RenderSizeChanged => "render_size_changed",
            Self::PipelineChanged => "pipeline_changed",
            Self::HistoryBindingChanged => "history_binding_changed",
            Self::FrameInputsChanged => "frame_inputs_changed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameHistoryStatus {
    pub current: Option<FrameHistoryHandle>,
    pub previous: Option<FrameHistoryHandle>,
    pub previous_available: bool,
    pub invalidation_reason: Option<FrameHistoryInvalidationReason>,
    pub target_size: UVec2,
    pub render_size: UVec2,
}

impl FrameHistoryStatus {
    pub const fn new(
        current: Option<FrameHistoryHandle>,
        previous: Option<FrameHistoryHandle>,
        previous_available: bool,
        invalidation_reason: Option<FrameHistoryInvalidationReason>,
        target_size: UVec2,
        render_size: UVec2,
    ) -> Self {
        Self {
            current,
            previous,
            previous_available,
            invalidation_reason,
            target_size,
            render_size,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHistoryCopyReport {
    pub history_target_present: bool,
    pub debug_marker_emitted: bool,
    pub target_size: UVec2,
    pub requested_copy_count: usize,
    pub copied_count: usize,
    pub scene_color_copied: bool,
    pub global_illumination_copied: bool,
    pub ambient_occlusion_copied: bool,
    pub screen_space_reflection_copied: bool,
    pub hzb_furthest_copied: bool,
    pub exposure_copied: bool,
}

impl RenderHistoryCopyReport {
    pub fn new(
        history_target_present: bool,
        target_size: UVec2,
        requested_copy_count: usize,
        scene_color_copied: bool,
        global_illumination_copied: bool,
        ambient_occlusion_copied: bool,
        screen_space_reflection_copied: bool,
        hzb_furthest_copied: bool,
        exposure_copied: bool,
    ) -> Self {
        Self {
            history_target_present,
            debug_marker_emitted: history_target_present && requested_copy_count > 0,
            target_size,
            requested_copy_count,
            copied_count: scene_color_copied as usize
                + global_illumination_copied as usize
                + ambient_occlusion_copied as usize
                + screen_space_reflection_copied as usize
                + hzb_furthest_copied as usize
                + exposure_copied as usize,
            scene_color_copied,
            global_illumination_copied,
            ambient_occlusion_copied,
            screen_space_reflection_copied,
            hzb_furthest_copied,
            exposure_copied,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetResolutionReport {
    pub target_kind: RenderCameraTargetKind,
    pub primary_target_size: UVec2,
    pub resolved_target_size: UVec2,
    pub effective_view_size: UVec2,
    pub effective_render_size: UVec2,
}

impl RenderCameraTargetResolutionReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        primary_target_size: UVec2,
        resolved_target_size: UVec2,
        effective_view_size: UVec2,
        effective_render_size: UVec2,
    ) -> Self {
        Self {
            target_kind,
            primary_target_size,
            resolved_target_size,
            effective_view_size,
            effective_render_size,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetWritebackStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForCopy,
    ReadyForConversion,
    SkippedDirectImport,
    Copied,
    Converted,
    BlockedFormatMismatch,
}

impl RenderCameraTargetWritebackStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::PendingTargetDescriptor => "pending_target_descriptor",
            Self::ReadyForCopy => "ready_for_copy",
            Self::ReadyForConversion => "ready_for_conversion",
            Self::SkippedDirectImport => "skipped_direct_import",
            Self::Copied => "copied",
            Self::Converted => "converted",
            Self::BlockedFormatMismatch => "blocked_format_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetWritebackReport {
    pub target_kind: RenderCameraTargetKind,
    pub status: RenderCameraTargetWritebackStatus,
    pub debug_marker_emitted: bool,
    pub conversion_debug_marker_emitted: bool,
    pub target_size: UVec2,
    pub copied_count: usize,
    pub converted_count: usize,
}

impl RenderCameraTargetWritebackReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        status: RenderCameraTargetWritebackStatus,
        debug_marker_emitted: bool,
        conversion_debug_marker_emitted: bool,
        target_size: UVec2,
        copied_count: usize,
        converted_count: usize,
    ) -> Self {
        Self {
            target_kind,
            status,
            debug_marker_emitted,
            conversion_debug_marker_emitted,
            target_size,
            copied_count,
            converted_count,
        }
    }

    pub fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self::new(
            target_kind,
            RenderCameraTargetWritebackStatus::NotRequested,
            false,
            false,
            UVec2::new(0, 0),
            0,
            0,
        )
    }

    pub const fn pending_target_descriptor(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::PendingTargetDescriptor,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn ready_for_copy(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::ReadyForCopy,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn ready_for_conversion(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::ReadyForConversion,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn skipped_direct_import(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::SkippedDirectImport,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn copied(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::Copied,
            true,
            false,
            target_size,
            1,
            0,
        )
    }

    pub const fn converted(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::Converted,
            false,
            true,
            target_size,
            0,
            1,
        )
    }

    pub const fn blocked_format_mismatch(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::BlockedFormatMismatch,
            false,
            false,
            target_size,
            0,
            0,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetGraphImportStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForDirectImport,
    DirectImported,
    RequiresConversionWriteback,
    BlockedFormatMismatch,
}

impl RenderCameraTargetGraphImportStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::PendingTargetDescriptor => "pending_target_descriptor",
            Self::ReadyForDirectImport => "ready_for_direct_import",
            Self::DirectImported => "direct_imported",
            Self::RequiresConversionWriteback => "requires_conversion_writeback",
            Self::BlockedFormatMismatch => "blocked_format_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetGraphImportReport {
    pub target_kind: RenderCameraTargetKind,
    pub status: RenderCameraTargetGraphImportStatus,
    pub target_size: UVec2,
    pub direct_import_count: usize,
    pub conversion_writeback_count: usize,
    pub blocked_count: usize,
}

impl RenderCameraTargetGraphImportReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        status: RenderCameraTargetGraphImportStatus,
        target_size: UVec2,
        direct_import_count: usize,
        conversion_writeback_count: usize,
        blocked_count: usize,
    ) -> Self {
        Self {
            target_kind,
            status,
            target_size,
            direct_import_count,
            conversion_writeback_count,
            blocked_count,
        }
    }

    pub fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self::new(
            target_kind,
            RenderCameraTargetGraphImportStatus::NotRequested,
            UVec2::new(0, 0),
            0,
            0,
            0,
        )
    }

    pub const fn pending_target_descriptor(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::PendingTargetDescriptor,
            target_size,
            0,
            0,
            0,
        )
    }

    pub const fn ready_for_direct_import(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::ReadyForDirectImport,
            target_size,
            0,
            0,
            0,
        )
    }

    pub const fn direct_imported(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::DirectImported,
            target_size,
            1,
            0,
            0,
        )
    }

    pub const fn requires_conversion_writeback(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
            target_size,
            0,
            1,
            0,
        )
    }

    pub const fn blocked_format_mismatch(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::BlockedFormatMismatch,
            target_size,
            0,
            0,
            1,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphTransientPoolReport {
    pub frame_index: u64,
    pub texture_created_count: usize,
    pub texture_reused_count: usize,
    pub buffer_created_count: usize,
    pub buffer_reused_count: usize,
    pub texture_pool_entry_count: usize,
    pub buffer_pool_entry_count: usize,
    pub evicted_texture_count: usize,
    pub evicted_buffer_count: usize,
}

impl RenderGraphTransientPoolReport {
    pub const fn new(
        frame_index: u64,
        texture_created_count: usize,
        texture_reused_count: usize,
        buffer_created_count: usize,
        buffer_reused_count: usize,
        texture_pool_entry_count: usize,
        buffer_pool_entry_count: usize,
        evicted_texture_count: usize,
        evicted_buffer_count: usize,
    ) -> Self {
        Self {
            frame_index,
            texture_created_count,
            texture_reused_count,
            buffer_created_count,
            buffer_reused_count,
            texture_pool_entry_count,
            buffer_pool_entry_count,
            evicted_texture_count,
            evicted_buffer_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionResourceReport {
    pub texture_view_count: usize,
    pub external_texture_view_count: usize,
    pub owned_texture_count: usize,
    pub buffer_count: usize,
    pub total_bound_resource_count: usize,
    pub transient_pool_report: RenderGraphTransientPoolReport,
}

impl RenderGraphExecutionResourceReport {
    pub const fn new(
        texture_view_count: usize,
        external_texture_view_count: usize,
        owned_texture_count: usize,
        buffer_count: usize,
    ) -> Self {
        Self {
            texture_view_count,
            external_texture_view_count,
            owned_texture_count,
            buffer_count,
            total_bound_resource_count: texture_view_count + buffer_count,
            transient_pool_report: RenderGraphTransientPoolReport::new(0, 0, 0, 0, 0, 0, 0, 0, 0),
        }
    }

    pub const fn with_transient_pool_report(
        mut self,
        transient_pool_report: RenderGraphTransientPoolReport,
    ) -> Self {
        self.transient_pool_report = transient_pool_report;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionCoverageReport {
    pub planned_live_pass_count: usize,
    pub executed_pass_count: usize,
    pub matched_planned_pass_count: usize,
    pub missing_planned_pass_count: usize,
    pub unexpected_executed_pass_count: usize,
    pub duplicate_executed_pass_count: usize,
}

impl RenderGraphExecutionCoverageReport {
    pub const fn new(
        planned_live_pass_count: usize,
        executed_pass_count: usize,
        matched_planned_pass_count: usize,
        missing_planned_pass_count: usize,
        unexpected_executed_pass_count: usize,
        duplicate_executed_pass_count: usize,
    ) -> Self {
        Self {
            planned_live_pass_count,
            executed_pass_count,
            matched_planned_pass_count,
            missing_planned_pass_count,
            unexpected_executed_pass_count,
            duplicate_executed_pass_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphStageExecutionReport {
    pub staged_pass_count: usize,
    pub unstaged_pass_count: usize,
    pub unique_stage_count: usize,
    pub stage_transition_count: usize,
    pub stage_order_violation_count: usize,
}

impl RenderGraphStageExecutionReport {
    pub const fn new(
        staged_pass_count: usize,
        unstaged_pass_count: usize,
        unique_stage_count: usize,
        stage_transition_count: usize,
        stage_order_violation_count: usize,
    ) -> Self {
        Self {
            staged_pass_count,
            unstaged_pass_count,
            unique_stage_count,
            stage_transition_count,
            stage_order_violation_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSceneVelocityReadbackReport {
    pub available: bool,
    pub size: UVec2,
    pub byte_len: usize,
    pub nonzero_pixel_count: usize,
}

impl RenderSceneVelocityReadbackReport {
    pub fn from_raw_rg16_float_bytes(size: UVec2, bytes: &[u8]) -> Self {
        let nonzero_pixel_count = bytes
            .chunks_exact(4)
            .filter(|pixel| pixel.iter().any(|byte| *byte != 0))
            .count();
        Self {
            available: true,
            size,
            byte_len: bytes.len(),
            nonzero_pixel_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MotionVectorCameraStatus {
    #[default]
    NotRequested,
    MissingPreviousCamera,
    CameraCutOrInvalid,
    Ready,
}

impl MotionVectorCameraStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::MissingPreviousCamera => "missing_previous_camera",
            Self::CameraCutOrInvalid => "camera_cut_or_invalid",
            Self::Ready => "ready",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphicsDebuggerStatus {
    /// True when the backend exposes a graphics-debugger capture hook through wgpu.
    /// This does not prove that RenderDoc or another debugger is attached.
    pub available: bool,
    /// Concrete backend selected by wgpu, for example `wgpu(dx12)` or `wgpu(vulkan)`.
    pub backend_name: String,
    pub capture_pending: bool,
    pub active_capture: bool,
    pub last_capture_frame: Option<u64>,
    pub last_error: Option<String>,
}

impl GraphicsDebuggerStatus {
    pub fn unavailable(backend_name: impl Into<String>) -> Self {
        Self {
            backend_name: backend_name.into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderQueueCapability {
    Graphics,
    Compute,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCapabilityKind {
    VirtualGeometry,
    HybridGlobalIllumination,
    AccelerationStructures,
    InlineRayQuery,
    RayTracingPipeline,
    BufferBindingArray,
    TextureBindingArray,
    NonUniformResourceIndexing,
    PartiallyBoundBindingArray,
    ScreenSpaceAntiAlias,
    StorageBuffers,
    IndirectDraw,
    BufferReadback,
    AsyncCompute,
    AsyncCopy,
    NeuralCompute,
    SparseTexture,
}

impl RenderCapabilityKind {
    pub const ALL: [Self; 17] = [
        Self::VirtualGeometry,
        Self::HybridGlobalIllumination,
        Self::AccelerationStructures,
        Self::InlineRayQuery,
        Self::RayTracingPipeline,
        Self::BufferBindingArray,
        Self::TextureBindingArray,
        Self::NonUniformResourceIndexing,
        Self::PartiallyBoundBindingArray,
        Self::ScreenSpaceAntiAlias,
        Self::StorageBuffers,
        Self::IndirectDraw,
        Self::BufferReadback,
        Self::AsyncCompute,
        Self::AsyncCopy,
        Self::NeuralCompute,
        Self::SparseTexture,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
            Self::AccelerationStructures => "acceleration_structures",
            Self::InlineRayQuery => "inline_ray_query",
            Self::RayTracingPipeline => "ray_tracing_pipeline",
            Self::BufferBindingArray => "buffer_binding_array",
            Self::TextureBindingArray => "texture_binding_array",
            Self::NonUniformResourceIndexing => "non_uniform_resource_indexing",
            Self::PartiallyBoundBindingArray => "partially_bound_binding_array",
            Self::ScreenSpaceAntiAlias => "screen_space_anti_alias",
            Self::StorageBuffers => "storage_buffers",
            Self::IndirectDraw => "indirect_draw",
            Self::BufferReadback => "buffer_readback",
            Self::AsyncCompute => "async_compute",
            Self::AsyncCopy => "async_copy",
            Self::NeuralCompute => "neural_compute",
            Self::SparseTexture => "sparse_texture",
        }
    }

    pub const fn capability_class(self) -> RenderCapabilityClass {
        match self {
            Self::ScreenSpaceAntiAlias => RenderCapabilityClass::Default,
            Self::VirtualGeometry
            | Self::HybridGlobalIllumination
            | Self::StorageBuffers
            | Self::IndirectDraw
            | Self::BufferReadback
            | Self::AsyncCompute
            | Self::AsyncCopy => RenderCapabilityClass::Advanced,
            Self::AccelerationStructures
            | Self::InlineRayQuery
            | Self::RayTracingPipeline
            | Self::BufferBindingArray
            | Self::TextureBindingArray
            | Self::NonUniformResourceIndexing
            | Self::PartiallyBoundBindingArray
            | Self::NeuralCompute
            | Self::SparseTexture => RenderCapabilityClass::Experimental,
        }
    }

    pub fn is_satisfied_by(self, capabilities: &RenderCapabilitySummary) -> bool {
        match self {
            Self::VirtualGeometry => capabilities.virtual_geometry_supported,
            Self::HybridGlobalIllumination => capabilities.hybrid_global_illumination_supported,
            Self::AccelerationStructures => capabilities.acceleration_structures_supported,
            Self::InlineRayQuery => capabilities.inline_ray_query,
            Self::RayTracingPipeline => capabilities.ray_tracing_pipeline,
            Self::BufferBindingArray => capabilities.supports_buffer_binding_array,
            Self::TextureBindingArray => capabilities.supports_texture_binding_array,
            Self::NonUniformResourceIndexing => capabilities.supports_non_uniform_resource_indexing,
            Self::PartiallyBoundBindingArray => capabilities.supports_partially_bound_binding_array,
            Self::ScreenSpaceAntiAlias => capabilities.supports_fxaa,
            Self::StorageBuffers => capabilities.supports_storage_buffers,
            Self::IndirectDraw => capabilities.supports_indirect_draw,
            Self::BufferReadback => capabilities.supports_buffer_readback,
            Self::AsyncCompute => capabilities.supports_async_compute,
            Self::AsyncCopy => capabilities.supports_async_copy,
            Self::NeuralCompute => capabilities.supports_neural_compute,
            Self::SparseTexture => capabilities.supports_sparse_texture,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCapabilityClass {
    Default,
    Advanced,
    Experimental,
}

impl RenderCapabilityClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Advanced => "advanced",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCapabilityMismatchDetail {
    pub capability: RenderCapabilityKind,
}

impl RenderCapabilityMismatchDetail {
    // Keep mismatch payloads backend-neutral so framework consumers never need graphics enums.
    pub const fn new(capability: RenderCapabilityKind) -> Self {
        Self { capability }
    }

    pub const fn label(self) -> &'static str {
        self.capability.label()
    }

    pub const fn capability_class(self) -> RenderCapabilityClass {
        self.capability.capability_class()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderCapabilitySummary {
    pub backend_name: String,
    pub queue_classes: Vec<RenderQueueCapability>,
    pub supports_surface: bool,
    pub supports_offscreen: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
    pub supports_pipeline_cache: bool,
    pub supports_storage_buffers: bool,
    pub max_storage_buffers_per_shader_stage: u32,
    pub supports_indirect_draw: bool,
    pub supports_multi_draw_indirect: bool,
    pub supports_indirect_first_instance: bool,
    pub supports_buffer_readback: bool,
    pub acceleration_structures_supported: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
    pub supports_buffer_binding_array: bool,
    pub supports_texture_binding_array: bool,
    pub supports_non_uniform_resource_indexing: bool,
    pub supports_partially_bound_binding_array: bool,
    pub supports_fxaa: bool,
    pub supports_smaa: bool,
    pub supports_taa: bool,
    pub supports_cas: bool,
    pub supports_dlss: bool,
    pub supports_neural_compute: bool,
    pub supports_sparse_texture: bool,
    pub max_supported_msaa_samples: u32,
    pub virtual_geometry_supported: bool,
    pub hybrid_global_illumination_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCapabilityClassReport {
    pub class: RenderCapabilityClass,
    pub satisfied: Vec<RenderCapabilityKind>,
    pub missing: Vec<RenderCapabilityMismatchDetail>,
}

impl RenderCapabilitySummary {
    pub const fn gpu_driven_submission_supported(&self) -> bool {
        self.supports_indirect_draw
            && self.supports_multi_draw_indirect
            && self.supports_indirect_first_instance
    }

    pub const fn storage_buffer_binding_capacity_supported(&self, required: u32) -> bool {
        self.max_storage_buffers_per_shader_stage >= required
    }

    pub const fn hzb_occlusion_culling_supported(
        &self,
        required_storage_buffers_per_shader_stage: u32,
    ) -> bool {
        self.supports_storage_buffers
            && self.storage_buffer_binding_capacity_supported(
                required_storage_buffers_per_shader_stage,
            )
            && self.gpu_driven_submission_supported()
    }

    pub fn capability_class_report(
        &self,
        class: RenderCapabilityClass,
    ) -> RenderCapabilityClassReport {
        let mut satisfied = Vec::new();
        let mut missing = Vec::new();

        for capability in RenderCapabilityKind::ALL {
            if capability.capability_class() != class {
                continue;
            }
            if capability.is_satisfied_by(self) {
                satisfied.push(capability);
            } else {
                missing.push(RenderCapabilityMismatchDetail::new(capability));
            }
        }

        RenderCapabilityClassReport {
            class,
            satisfied,
            missing,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    SubmitFrameExtract {
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    },
    ReloadPipeline {
        pipeline: RenderPipelineHandle,
    },
    SetQualityProfile {
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderQuery {
    Stats,
    CaptureFrame(RenderViewportHandle),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderHybridGiPayloadSource {
    #[default]
    None,
    Authored,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderVirtualGeometryPayloadSource {
    #[default]
    None,
    Authored,
    AutomaticFallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderViewportDescriptor {
    pub size: UVec2,
    pub label: Option<String>,
}

impl RenderViewportDescriptor {
    pub fn new(size: UVec2) -> Self {
        Self { size, label: None }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureQualitySettings {
    pub clustered_lighting: bool,
    pub screen_space_ambient_occlusion: bool,
    pub temporal_history: bool,
    pub bloom: bool,
    pub color_grading: bool,
    pub anti_alias: bool,
    pub reflection_probes: bool,
    pub baked_lighting: bool,
    pub particle_rendering: bool,
    pub virtual_geometry: bool,
    pub hybrid_global_illumination: bool,
    pub solari: bool,
    pub allow_async_compute: bool,
}

impl Default for RenderFeatureQualitySettings {
    fn default() -> Self {
        Self {
            clustered_lighting: true,
            screen_space_ambient_occlusion: true,
            temporal_history: false,
            bloom: true,
            color_grading: true,
            anti_alias: true,
            reflection_probes: true,
            baked_lighting: true,
            particle_rendering: true,
            virtual_geometry: false,
            hybrid_global_illumination: false,
            solari: false,
            allow_async_compute: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderQualityProfile {
    pub name: String,
    pub pipeline_override: Option<RenderPipelineHandle>,
    pub features: RenderFeatureQualitySettings,
    pub taa_quality: TaaQualityPreset,
    pub solari: SolariSettings,
}

impl RenderQualityProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pipeline_override: None,
            features: RenderFeatureQualitySettings::default(),
            taa_quality: TaaQualityPreset::default(),
            solari: SolariSettings::default(),
        }
    }

    pub fn with_pipeline_asset(mut self, pipeline: RenderPipelineHandle) -> Self {
        self.pipeline_override = Some(pipeline);
        self
    }

    pub fn with_clustered_lighting(mut self, enabled: bool) -> Self {
        self.features.clustered_lighting = enabled;
        self
    }

    pub fn with_screen_space_ambient_occlusion(mut self, enabled: bool) -> Self {
        self.features.screen_space_ambient_occlusion = enabled;
        self
    }

    pub fn with_temporal_history(mut self, enabled: bool) -> Self {
        self.features.temporal_history = enabled;
        self
    }

    pub fn with_bloom(mut self, enabled: bool) -> Self {
        self.features.bloom = enabled;
        self
    }

    pub fn with_color_grading(mut self, enabled: bool) -> Self {
        self.features.color_grading = enabled;
        self
    }

    pub fn with_anti_alias(mut self, enabled: bool) -> Self {
        self.features.anti_alias = enabled;
        self
    }

    pub fn with_taa_quality(mut self, quality: TaaQualityPreset) -> Self {
        self.taa_quality = quality;
        self
    }

    pub fn with_reflection_probes(mut self, enabled: bool) -> Self {
        self.features.reflection_probes = enabled;
        self
    }

    pub fn with_baked_lighting(mut self, enabled: bool) -> Self {
        self.features.baked_lighting = enabled;
        self
    }

    pub fn with_particle_rendering(mut self, enabled: bool) -> Self {
        self.features.particle_rendering = enabled;
        self
    }

    pub fn with_virtual_geometry(mut self, enabled: bool) -> Self {
        self.features.virtual_geometry = enabled;
        self
    }

    pub fn with_hybrid_global_illumination(mut self, enabled: bool) -> Self {
        self.features.hybrid_global_illumination = enabled;
        self
    }

    pub fn with_solari(mut self, enabled: bool) -> Self {
        self.features.solari = enabled;
        self
    }

    pub fn with_solari_settings(mut self, settings: SolariSettings) -> Self {
        self.solari = settings;
        self
    }

    pub fn with_solari_experimental_enabled(mut self, enabled: bool) -> Self {
        self.solari = if enabled {
            SolariSettings::experimental_enabled()
        } else {
            SolariSettings::default()
        };
        self
    }

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.features.allow_async_compute = enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderGpuSceneUploadPath {
    #[default]
    DirectQueueWrite,
}

impl RenderGpuSceneUploadPath {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DirectQueueWrite => "direct_queue_write",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub active_viewports: usize,
    pub submitted_frames: u64,
    pub captured_frames: u64,
    pub last_generation: Option<u64>,
    pub last_pipeline: Option<RenderPipelineHandle>,
    pub last_frame_target_size: Option<UVec2>,
    pub last_frame_render_size: Option<UVec2>,
    pub last_frame_history: Option<FrameHistoryHandle>,
    pub last_frame_history_status: FrameHistoryStatus,
    pub last_frame_history_copy_report: RenderHistoryCopyReport,
    pub last_camera_target_resolution: RenderCameraTargetResolutionReport,
    pub last_camera_target_graph_import: RenderCameraTargetGraphImportReport,
    pub last_camera_target_writeback: RenderCameraTargetWritebackReport,
    pub last_capture_report: super::RenderCaptureReport,
    pub last_scene_camera_scheduled_count: usize,
    pub last_scene_camera_order_ambiguity_count: usize,
    pub last_visibility_view_count: usize,
    pub last_visibility_input_count: usize,
    pub last_visibility_layer_filtered_count: usize,
    pub last_visibility_frustum_culled_count: usize,
    pub last_visibility_occlusion_culled_count: usize,
    pub last_visibility_visible_count: usize,
    pub last_visibility_static_index_full_rebuild_count: usize,
    pub last_visibility_static_index_incremental_update_count: usize,
    pub last_visibility_static_index_inserted_count: usize,
    pub last_visibility_static_index_updated_count: usize,
    pub last_visibility_static_index_removed_count: usize,
    pub last_visibility_static_index_indexed_entity_count: usize,
    pub last_visibility_static_index_occupied_cell_count: usize,
    pub last_visibility_static_index_main_view_prefilter_used: bool,
    pub last_visibility_static_index_main_view_static_input_count: usize,
    pub last_visibility_static_index_main_view_static_candidate_count: usize,
    pub last_hzb_mip_count: usize,
    pub last_hzb_graph_executed_pass_count: usize,
    pub last_hzb_occlusion_reported: bool,
    pub last_hzb_occlusion_candidate_arg_count: usize,
    pub last_hzb_occlusion_candidate_instance_count: usize,
    pub last_hzb_occlusion_dispatch_group_count: usize,
    pub last_hzb_occlusion_dispatched_phase_count: usize,
    pub last_hzb_occlusion_history_available: bool,
    pub last_hzb_occlusion_readback_available: bool,
    pub last_hzb_occlusion_tested_arg_count: usize,
    pub last_hzb_occlusion_tested_instance_count: usize,
    pub last_hzb_occlusion_culled_arg_count: usize,
    pub last_hzb_occlusion_culled_instance_count: usize,
    pub last_hzb_occlusion_indirect_args_readback_available: bool,
    pub last_hzb_occlusion_readback_arg_count: usize,
    pub last_hzb_occlusion_compacted_draw_count: usize,
    pub last_hzb_occlusion_zero_instance_arg_count: usize,
    pub last_hzb_occlusion_remaining_instance_count: usize,
    pub last_light_grid_reported: bool,
    pub last_light_grid_light_count: usize,
    pub last_light_grid_tile_count: usize,
    pub last_light_grid_zbin_count: usize,
    pub last_light_grid_non_empty_tile_count: usize,
    pub last_light_grid_non_empty_zbin_count: usize,
    pub last_light_grid_non_empty_cluster_count: usize,
    pub last_light_grid_peak_lights_per_cluster: usize,
    pub last_light_grid_average_lights_per_cluster_milli: usize,
    pub last_quality_profile: Option<String>,
    pub last_effective_features: Vec<String>,
    pub last_graph_pass_count: usize,
    pub last_graph_culled_pass_count: usize,
    pub last_graph_queue_fallback_pass_count: usize,
    pub last_graph_resource_lifetime_count: usize,
    pub last_graph_sparse_texture_lifetime_count: usize,
    pub last_graph_planned_resource_access_count: usize,
    pub last_graph_planned_dependency_count: usize,
    pub last_graph_transient_texture_slot_count: usize,
    pub last_graph_sparse_texture_slot_count: usize,
    pub last_graph_transient_buffer_slot_count: usize,
    pub last_graph_transient_texture_bytes_reserved: u64,
    pub last_graph_transient_buffer_bytes_reserved: u64,
    pub last_graph_transient_dense_bytes_reserved: u64,
    pub last_graph_sparse_texture_virtual_bytes: u64,
    pub last_graph_executed_pass_count: usize,
    pub last_graph_executed_passes: Vec<String>,
    pub last_graph_executed_executor_ids: Vec<String>,
    pub last_graph_executed_debug_markers: Vec<String>,
    pub last_graph_executed_resource_access_count: usize,
    pub last_graph_executed_dependency_count: usize,
    pub last_graph_compute_dispatch_count: usize,
    pub last_graph_compute_dispatch_group_count: usize,
    pub last_graph_compute_storage_write_resource_count: usize,
    pub last_graph_compute_planned_workload_count: usize,
    pub last_graph_compute_matched_workload_count: usize,
    pub last_graph_compute_missing_dispatch_count: usize,
    pub last_graph_compute_workload_mismatch_count: usize,
    pub last_graph_compute_unexpected_dispatch_count: usize,
    pub last_graph_execution_resource_report: RenderGraphExecutionResourceReport,
    pub last_graph_execution_coverage_report: RenderGraphExecutionCoverageReport,
    pub last_graph_stage_execution_report: RenderGraphStageExecutionReport,
    pub last_scene_velocity_readback_report: RenderSceneVelocityReadbackReport,
    pub last_post_process_graph_node_count: usize,
    pub last_post_process_graph_skipped_node_count: usize,
    pub last_post_process_output_transfer_node: Option<String>,
    pub last_post_process_graph_executed_nodes: Vec<String>,
    pub last_post_process_effect_stack_report: RenderPostProcessEffectStackReport,
    pub last_post_process_lut_request_count: usize,
    pub last_post_process_lut_ready_count: usize,
    pub last_post_process_lut_fallback_count: usize,
    pub last_post_process_lut_2d_strip_ready_count: usize,
    pub last_post_process_lut_3d_request_count: usize,
    pub last_post_process_lut_unsupported_shape_count: usize,
    pub last_motion_vector_camera_status: MotionVectorCameraStatus,
    pub last_anti_alias_fallback: AntiAliasFallbackReport,
    pub last_graph_requested_msaa_sample_count: u32,
    pub last_graph_effective_msaa_sample_count: u32,
    pub last_anti_alias_graph_executed_pass_count: usize,
    pub last_virtual_geometry_graph_executed_pass_count: usize,
    pub last_hybrid_gi_graph_executed_pass_count: usize,
    pub last_particle_graph_executed_pass_count: usize,
    pub last_shadow_graph_executed_pass_count: usize,
    pub last_shadow_execution_report: RenderShadowExecutionReport,
    pub last_transparent_graph_executed_pass_count: usize,
    pub last_particle_velocity_missing_sprite_count: usize,
    pub last_particle_velocity_anonymous_stream_ambiguity_count: usize,
    pub last_particle_gpu_alive_count: usize,
    pub last_particle_gpu_spawned_total: usize,
    pub last_particle_gpu_emitter_readback_count: usize,
    pub last_particle_gpu_indirect_instance_count: usize,
    pub last_async_compute_pass_count: usize,
    pub last_ui_command_count: usize,
    pub last_ui_quad_count: usize,
    pub last_ui_text_payload_count: usize,
    pub last_ui_image_payload_count: usize,
    pub last_ui_clipped_command_count: usize,
    pub last_ui_graph_executed_pass_count: usize,
    pub last_ui_target_size: Option<UVec2>,
    pub last_ui_graph_pass_order: Option<String>,
    pub last_material_count: usize,
    pub last_material_ready_count: usize,
    pub last_material_fallback_count: usize,
    pub last_material_validation_error_count: usize,
    pub last_material_diagnostic_count: usize,
    pub last_mesh_draw_count: usize,
    pub last_mesh_opaque_draw_count: usize,
    pub last_mesh_alpha_mask_draw_count: usize,
    pub last_mesh_transparent_draw_count: usize,
    pub last_mesh_early_z_draw_count: usize,
    pub last_mesh_shadow_caster_draw_count: usize,
    pub last_mesh_alpha_mask_shadow_caster_draw_count: usize,
    pub last_mesh_prepared_geometry_draw_count: usize,
    pub last_mesh_dynamic_geometry_draw_count: usize,
    pub last_mesh_skinned_draw_count: usize,
    pub last_mesh_skinned_palette_upload_count: usize,
    pub last_mesh_skinned_previous_palette_upload_count: usize,
    pub last_mesh_skinned_gpu_source_candidate_count: usize,
    pub last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: usize,
    pub last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: usize,
    pub last_mesh_skinned_gpu_skinning_draw_count: usize,
    pub last_mesh_skinned_gpu_velocity_draw_count: usize,
    pub last_mesh_indirect_draw_count: usize,
    pub last_mesh_lod_draw_count: usize,
    pub last_mesh_previous_velocity_transform_draw_count: usize,
    pub last_mesh_missing_velocity_transform_draw_count: usize,
    pub last_mesh_taa_reactive_mask_command_count: usize,
    pub last_mesh_static_batch_candidate_group_count: usize,
    pub last_mesh_static_batch_candidate_draw_count: usize,
    pub last_mesh_dynamic_batch_candidate_group_count: usize,
    pub last_mesh_dynamic_batch_candidate_draw_count: usize,
    pub last_mesh_gpu_instancing_candidate_group_count: usize,
    pub last_mesh_gpu_instancing_candidate_draw_count: usize,
    pub last_indirect_batch_count: usize,
    pub last_indirect_batched_draw_count: usize,
    pub last_indirect_fallback_draw_count: usize,
    pub last_indirect_args_count: usize,
    pub last_gpu_scene_primitive_count: u32,
    pub last_gpu_scene_instance_count: u32,
    pub last_gpu_scene_dirty_entry_count: usize,
    pub last_gpu_scene_uploaded_bytes: u64,
    pub last_gpu_scene_upload_path: RenderGpuSceneUploadPath,
    pub last_gpu_scene_free_span_count: usize,
    pub last_gpu_scene_primitive_upload_range_count: usize,
    pub last_gpu_scene_instance_upload_range_count: usize,
    pub last_sprite_count: usize,
    pub last_sprite_ready_count: usize,
    pub last_sprite_texture_fallback_count: usize,
    pub last_sprite_graph_executed_pass_count: usize,
    pub last_sprite_draw_batch_count: usize,
    pub last_sprite_batched_sprite_count: usize,
    pub last_sprite_image_slice_count: usize,
    pub last_sprite_expanded_image_slice_count: usize,
    pub last_sprite_vertex_count: usize,
    pub last_sprite_opaque_draw_batch_count: usize,
    pub last_sprite_alpha_mask_draw_batch_count: usize,
    pub last_sprite_transparent_draw_batch_count: usize,
    pub last_directional_light_count: usize,
    pub last_directional_light_ready_count: usize,
    pub last_directional_light_degraded_count: usize,
    pub last_point_light_count: usize,
    pub last_point_light_ready_count: usize,
    pub last_point_light_degraded_count: usize,
    pub last_spot_light_count: usize,
    pub last_spot_light_ready_count: usize,
    pub last_spot_light_degraded_count: usize,
    pub last_ambient_light_count: usize,
    pub last_ambient_light_ready_count: usize,
    pub last_ambient_light_degraded_count: usize,
    pub last_rect_light_count: usize,
    pub last_rect_light_ready_count: usize,
    pub last_rect_light_degraded_count: usize,
    pub last_virtual_geometry_cluster_budget: usize,
    pub last_virtual_geometry_page_budget: usize,
    pub last_virtual_geometry_input_cluster_count: usize,
    pub last_virtual_geometry_input_page_count: usize,
    pub last_virtual_geometry_visible_cluster_count: usize,
    pub last_virtual_geometry_visible_entity_count: usize,
    pub last_virtual_geometry_instance_count: usize,
    pub last_virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    pub last_virtual_geometry_requested_page_count: usize,
    pub last_virtual_geometry_dirty_page_count: usize,
    pub last_virtual_geometry_forced_mip: Option<u8>,
    pub last_virtual_geometry_freeze_cull: bool,
    pub last_virtual_geometry_visualize_bvh: bool,
    pub last_virtual_geometry_visualize_visbuffer: bool,
    pub last_virtual_geometry_print_leaf_clusters: bool,
    pub last_virtual_geometry_page_table_entry_count: usize,
    pub last_virtual_geometry_resident_page_count: usize,
    pub last_virtual_geometry_pending_request_count: usize,
    pub last_virtual_geometry_page_dependency_count: usize,
    pub last_virtual_geometry_completed_page_count: usize,
    pub last_virtual_geometry_replaced_page_count: usize,
    pub last_virtual_geometry_indirect_draw_count: usize,
    pub last_virtual_geometry_indirect_buffer_count: usize,
    pub last_virtual_geometry_indirect_args_count: usize,
    pub last_virtual_geometry_indirect_segment_count: usize,
    pub last_virtual_geometry_execution_segment_count: usize,
    pub last_virtual_geometry_execution_page_count: usize,
    pub last_virtual_geometry_execution_resident_segment_count: usize,
    pub last_virtual_geometry_execution_pending_segment_count: usize,
    pub last_virtual_geometry_execution_missing_segment_count: usize,
    pub last_virtual_geometry_execution_repeated_draw_count: usize,
    pub last_virtual_geometry_cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub last_virtual_geometry_node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub last_virtual_geometry_node_and_cluster_cull_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_dispatch_group_count: [usize; 3],
    pub last_virtual_geometry_node_and_cluster_cull_instance_seed_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_instance_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_child_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_traversal_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_page_request_count: usize,
    pub last_virtual_geometry_selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    pub last_virtual_geometry_selected_cluster_count: usize,
    pub last_virtual_geometry_visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub last_virtual_geometry_visbuffer64_entry_count: usize,
    pub last_virtual_geometry_hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub last_virtual_geometry_hardware_rasterization_record_count: usize,
    pub last_hybrid_gi_active_probe_count: usize,
    pub last_hybrid_gi_requested_probe_count: usize,
    pub last_hybrid_gi_dirty_probe_count: usize,
    pub last_hybrid_gi_cache_entry_count: usize,
    pub last_hybrid_gi_resident_probe_count: usize,
    pub last_hybrid_gi_pending_update_count: usize,
    pub last_hybrid_gi_scheduled_trace_region_count: usize,
    pub last_hybrid_gi_scene_card_count: usize,
    pub last_hybrid_gi_scene_screen_probe_count: usize,
    pub last_hybrid_gi_scene_radiance_cache_entry_count: usize,
    pub last_hybrid_gi_surface_cache_resident_page_count: usize,
    pub last_hybrid_gi_surface_cache_dirty_page_count: usize,
    pub last_hybrid_gi_surface_cache_feedback_card_count: usize,
    pub last_hybrid_gi_surface_cache_capture_slot_count: usize,
    pub last_hybrid_gi_surface_cache_invalidated_page_count: usize,
    pub last_hybrid_gi_voxel_resident_clipmap_count: usize,
    pub last_hybrid_gi_voxel_dirty_clipmap_count: usize,
    pub last_hybrid_gi_voxel_invalidated_clipmap_count: usize,
    pub last_hybrid_gi_payload_source: RenderHybridGiPayloadSource,
    pub capabilities: RenderCapabilitySummary,
    pub advanced_provider_availability: AdvancedProviderAvailability,
    pub last_advanced_provider_reports: Vec<AdvancedProviderReport>,
    pub last_solari_runtime_report: SolariRuntimeReport,
}

#[cfg(test)]
mod tests {
    use crate::core::math::UVec2;

    use super::{
        RenderCameraTargetWritebackReport, RenderCapabilityClass, RenderCapabilityKind,
        RenderCapabilityMismatchDetail, RenderCapabilitySummary,
        RenderGraphExecutionCoverageReport, RenderGraphStageExecutionReport,
        RenderHistoryCopyReport, RenderQualityProfile, TaaQualityPreset,
    };

    #[test]
    fn history_copy_report_counts_copied_slots_from_slot_flags() {
        let report = RenderHistoryCopyReport::new(
            true,
            UVec2::new(960, 540),
            6,
            true,
            true,
            false,
            true,
            true,
            true,
        );

        assert!(report.history_target_present);
        assert!(report.debug_marker_emitted);
        assert_eq!(report.target_size, UVec2::new(960, 540));
        assert_eq!(report.requested_copy_count, 6);
        assert_eq!(report.copied_count, 5);

        let missing_target_report = RenderHistoryCopyReport::new(
            false,
            UVec2::new(960, 540),
            1,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert!(!missing_target_report.debug_marker_emitted);
    }

    #[test]
    fn camera_target_writeback_report_separates_copy_and_conversion_debug_markers() {
        let size = UVec2::new(72, 40);
        let copied = RenderCameraTargetWritebackReport::copied(size);
        let ready = RenderCameraTargetWritebackReport::ready_for_copy(size);
        let converted = RenderCameraTargetWritebackReport::converted(size);
        let blocked = RenderCameraTargetWritebackReport::blocked_format_mismatch(size);

        assert!(copied.debug_marker_emitted);
        assert!(!copied.conversion_debug_marker_emitted);
        assert_eq!(copied.copied_count, 1);
        assert_eq!(converted.converted_count, 1);
        assert_eq!(converted.copied_count, 0);
        assert!(!converted.debug_marker_emitted);
        assert!(converted.conversion_debug_marker_emitted);
        assert!(!ready.debug_marker_emitted);
        assert!(!ready.conversion_debug_marker_emitted);
        assert!(!blocked.debug_marker_emitted);
        assert!(!blocked.conversion_debug_marker_emitted);
    }

    #[test]
    fn graph_stage_execution_report_preserves_neutral_counts() {
        let report = RenderGraphStageExecutionReport::new(8, 2, 5, 4, 1);

        assert_eq!(report.staged_pass_count, 8);
        assert_eq!(report.unstaged_pass_count, 2);
        assert_eq!(report.unique_stage_count, 5);
        assert_eq!(report.stage_transition_count, 4);
        assert_eq!(report.stage_order_violation_count, 1);
    }

    #[test]
    fn graph_execution_coverage_report_preserves_neutral_counts() {
        let report = RenderGraphExecutionCoverageReport::new(14, 15, 13, 1, 2, 1);

        assert_eq!(report.planned_live_pass_count, 14);
        assert_eq!(report.executed_pass_count, 15);
        assert_eq!(report.matched_planned_pass_count, 13);
        assert_eq!(report.missing_planned_pass_count, 1);
        assert_eq!(report.unexpected_executed_pass_count, 2);
        assert_eq!(report.duplicate_executed_pass_count, 1);
    }

    #[test]
    fn render_quality_profile_preserves_taa_quality_preset() {
        let profile =
            RenderQualityProfile::new("taa-high").with_taa_quality(TaaQualityPreset::High);

        assert_eq!(profile.taa_quality, TaaQualityPreset::High);
        assert_eq!(
            RenderQualityProfile::new("default").taa_quality,
            TaaQualityPreset::Medium
        );
    }

    #[test]
    fn capability_class_report_splits_default_advanced_and_experimental_requirements() {
        let capabilities = RenderCapabilitySummary {
            backend_name: "class-report-test".to_string(),
            supports_fxaa: true,
            virtual_geometry_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            acceleration_structures_supported: true,
            supports_buffer_binding_array: true,
            supports_texture_binding_array: true,
            ..RenderCapabilitySummary::default()
        };

        let default = capabilities.capability_class_report(RenderCapabilityClass::Default);
        assert_eq!(
            default.satisfied,
            vec![RenderCapabilityKind::ScreenSpaceAntiAlias]
        );
        assert!(default.missing.is_empty());

        let advanced = capabilities.capability_class_report(RenderCapabilityClass::Advanced);
        assert_eq!(
            advanced.satisfied,
            vec![
                RenderCapabilityKind::VirtualGeometry,
                RenderCapabilityKind::StorageBuffers,
                RenderCapabilityKind::IndirectDraw,
                RenderCapabilityKind::BufferReadback,
            ]
        );
        assert_eq!(
            advanced.missing,
            vec![
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::HybridGlobalIllumination,),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCompute),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCopy),
            ]
        );

        let experimental =
            capabilities.capability_class_report(RenderCapabilityClass::Experimental);
        assert_eq!(
            experimental.satisfied,
            vec![
                RenderCapabilityKind::AccelerationStructures,
                RenderCapabilityKind::BufferBindingArray,
                RenderCapabilityKind::TextureBindingArray,
            ]
        );
        assert_eq!(
            experimental.missing,
            vec![
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::InlineRayQuery),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::RayTracingPipeline),
                RenderCapabilityMismatchDetail::new(
                    RenderCapabilityKind::NonUniformResourceIndexing,
                ),
                RenderCapabilityMismatchDetail::new(
                    RenderCapabilityKind::PartiallyBoundBindingArray,
                ),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::NeuralCompute),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::SparseTexture),
            ]
        );
    }

    #[test]
    fn gpu_driven_submission_requires_indirect_multi_draw_and_first_instance() {
        let supported = RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        };
        assert!(supported.gpu_driven_submission_supported());

        for capabilities in [
            RenderCapabilitySummary {
                supports_multi_draw_indirect: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_indirect_draw: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_indirect_draw: true,
                supports_multi_draw_indirect: true,
                ..RenderCapabilitySummary::default()
            },
        ] {
            assert!(!capabilities.gpu_driven_submission_supported());
        }
    }

    #[test]
    fn hzb_occlusion_culling_requires_storage_buffers_gpu_driven_and_binding_capacity() {
        const REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 10;

        let supported = RenderCapabilitySummary {
            supports_storage_buffers: true,
            max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        };
        assert!(
            supported.hzb_occlusion_culling_supported(REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE)
        );

        for capabilities in [
            RenderCapabilitySummary {
                max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                supports_indirect_draw: true,
                supports_multi_draw_indirect: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_storage_buffers: true,
                max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                supports_multi_draw_indirect: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_storage_buffers: true,
                max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                supports_indirect_draw: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_storage_buffers: true,
                max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
                supports_indirect_draw: true,
                supports_multi_draw_indirect: true,
                ..RenderCapabilitySummary::default()
            },
            RenderCapabilitySummary {
                supports_storage_buffers: true,
                max_storage_buffers_per_shader_stage: REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
                supports_indirect_draw: true,
                supports_multi_draw_indirect: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
        ] {
            assert!(!capabilities
                .hzb_occlusion_culling_supported(REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE));
        }
    }
}
