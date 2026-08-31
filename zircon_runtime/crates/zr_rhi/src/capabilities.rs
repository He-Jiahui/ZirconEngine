use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderQueueClass {
    Graphics,
    Compute,
    Copy,
}

/// An operation that can be admitted through the neutral RHI contract.
///
/// This is deliberately distinct from adapter feature bits: a backend must not
/// advertise an operation here until the neutral command or service surface can
/// execute it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RenderOperation {
    DirectDraw,
    IndexedDraw,
    ComputeDispatch,
    BufferToBufferCopy,
    BufferToTextureCopy,
    TextureToBufferCopy,
    DebugMarker,
    DebugGroup,
    IndirectDraw,
    MultiDrawIndirect,
    MultiDrawIndirectCount,
    AsyncComputeQueue,
    AsyncCopyQueue,
    GraphicsDebuggerCapture,
    TextureToTextureCopy,
    /// Compute dispatch whose workgroup dimensions are sourced from an
    /// indirect-argument buffer.
    ComputeDispatchIndirect,
}

impl RenderOperation {
    /// Append only: serialized operation matrices use these discriminant indices.
    pub const COUNT: usize = Self::ComputeDispatchIndirect as usize + 1;

    pub const ALL: [Self; Self::COUNT] = [
        Self::DirectDraw,
        Self::IndexedDraw,
        Self::ComputeDispatch,
        Self::BufferToBufferCopy,
        Self::BufferToTextureCopy,
        Self::TextureToBufferCopy,
        Self::DebugMarker,
        Self::DebugGroup,
        Self::IndirectDraw,
        Self::MultiDrawIndirect,
        Self::MultiDrawIndirectCount,
        Self::AsyncComputeQueue,
        Self::AsyncCopyQueue,
        Self::GraphicsDebuggerCapture,
        Self::TextureToTextureCopy,
        Self::ComputeDispatchIndirect,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

/// Whether a backend can execute a neutral RHI operation without or with a
/// documented emulation path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderOperationSupport {
    #[default]
    Unsupported,
    Emulated,
    Native,
}

impl RenderOperationSupport {
    pub const fn is_supported(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

/// Structured reason returned when a neutral RHI operation has no executable
/// backend path. This keeps admission failures allocation-free and machine
/// readable for callers that need to select a fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedRenderOperation {
    pub operation: RenderOperation,
    pub support: RenderOperationSupport,
}

/// Fixed-size operation table so admission stays allocation-free and every
/// operation has an explicit fail-closed default.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderOperationMatrix {
    support: [RenderOperationSupport; RenderOperation::COUNT],
}

impl Default for RenderOperationMatrix {
    fn default() -> Self {
        Self {
            support: [RenderOperationSupport::Unsupported; RenderOperation::COUNT],
        }
    }
}

impl RenderOperationMatrix {
    pub const fn support(&self, operation: RenderOperation) -> RenderOperationSupport {
        self.support[operation.index()]
    }

    pub fn set(&mut self, operation: RenderOperation, support: RenderOperationSupport) {
        self.support[operation.index()] = support;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccelerationStructureCaps {
    pub supported: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
    pub max_instance_count: Option<u32>,
}

impl AccelerationStructureCaps {
    pub fn disabled() -> Self {
        Self {
            supported: false,
            inline_ray_query: false,
            ray_tracing_pipeline: false,
            max_instance_count: None,
        }
    }

    pub fn basic(max_instance_count: u32) -> Self {
        Self {
            supported: true,
            inline_ray_query: false,
            ray_tracing_pipeline: false,
            max_instance_count: Some(max_instance_count),
        }
    }
}

/// Stable identity of the adapter that created a render device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderAdapterInfo {
    pub name: String,
    pub device_type: String,
}

/// Actual limits negotiated for the render device, rather than requested limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceLimits {
    pub max_bind_groups: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_array_layers: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    #[serde(default)]
    pub max_binding_array_elements_per_shader_stage: u32,
    #[serde(default)]
    pub max_binding_array_sampler_elements_per_shader_stage: u32,
    #[serde(default = "default_dynamic_buffer_offset_alignment")]
    pub min_uniform_buffer_offset_alignment: u32,
    #[serde(default = "default_dynamic_buffer_offset_alignment")]
    pub min_storage_buffer_offset_alignment: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_buffer_binding_size: u64,
}

const fn default_dynamic_buffer_offset_alignment() -> u32 {
    256
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderBackendCaps {
    pub backend_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<RenderAdapterInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_limits: Option<RenderDeviceLimits>,
    pub queue_classes: Vec<RenderQueueClass>,
    pub supports_surface: bool,
    pub supports_offscreen: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
    pub supports_pipeline_cache: bool,
    #[serde(default)]
    pub supports_gpu_timestamp: bool,
    #[serde(default)]
    pub supports_subgroup: bool,
    #[serde(default)]
    pub supports_pipeline_statistics_query: bool,
    pub supports_storage_buffers: bool,
    #[serde(default)]
    pub supports_fragment_writable_storage: bool,
    #[serde(default)]
    pub max_storage_buffers_per_shader_stage: u32,
    #[serde(default)]
    pub max_storage_buffer_binding_size: u64,
    pub supports_indirect_draw: bool,
    pub supports_multi_draw_indirect: bool,
    #[serde(default)]
    pub supports_multi_draw_indirect_count: bool,
    pub supports_indirect_first_instance: bool,
    pub supports_buffer_readback: bool,
    pub supports_buffer_binding_array: bool,
    pub supports_texture_binding_array: bool,
    pub supports_non_uniform_resource_indexing: bool,
    pub supports_partially_bound_binding_array: bool,
    pub supports_neural_compute: bool,
    pub supports_sparse_texture: bool,
    pub supports_debug_markers: bool,
    pub supports_debug_groups: bool,
    pub supports_graphics_debugger_capture: bool,
    pub acceleration_structures: AccelerationStructureCaps,
    #[serde(default)]
    pub operation_matrix: RenderOperationMatrix,
}

impl RenderBackendCaps {
    pub fn new(backend_name: impl Into<String>) -> Self {
        Self {
            backend_name: backend_name.into(),
            adapter: None,
            device_limits: None,
            queue_classes: Vec::new(),
            supports_surface: false,
            supports_offscreen: true,
            supports_async_compute: false,
            supports_async_copy: false,
            supports_pipeline_cache: false,
            supports_gpu_timestamp: false,
            supports_subgroup: false,
            supports_pipeline_statistics_query: false,
            supports_storage_buffers: false,
            supports_fragment_writable_storage: false,
            max_storage_buffers_per_shader_stage: 0,
            max_storage_buffer_binding_size: 0,
            supports_indirect_draw: false,
            supports_multi_draw_indirect: false,
            supports_multi_draw_indirect_count: false,
            supports_indirect_first_instance: false,
            supports_buffer_readback: false,
            supports_buffer_binding_array: false,
            supports_texture_binding_array: false,
            supports_non_uniform_resource_indexing: false,
            supports_partially_bound_binding_array: false,
            supports_neural_compute: false,
            supports_sparse_texture: false,
            supports_debug_markers: false,
            supports_debug_groups: false,
            supports_graphics_debugger_capture: false,
            acceleration_structures: AccelerationStructureCaps::disabled(),
            operation_matrix: RenderOperationMatrix::default(),
        }
    }

    pub fn with_adapter(mut self, adapter: RenderAdapterInfo) -> Self {
        self.adapter = Some(adapter);
        self
    }

    pub fn with_device_limits(mut self, device_limits: RenderDeviceLimits) -> Self {
        self.device_limits = Some(device_limits);
        self
    }

    pub fn with_queue(mut self, queue: RenderQueueClass) -> Self {
        if !self.queue_classes.contains(&queue) {
            self.queue_classes.push(queue);
        }
        self
    }

    pub fn supports_queue(&self, queue: RenderQueueClass) -> bool {
        self.queue_classes.contains(&queue)
    }

    pub fn with_operation_support(
        mut self,
        operation: RenderOperation,
        support: RenderOperationSupport,
    ) -> Self {
        self.operation_matrix.set(operation, support);
        self
    }

    pub const fn operation_support(&self, operation: RenderOperation) -> RenderOperationSupport {
        self.operation_matrix.support(operation)
    }

    pub const fn supports_operation(&self, operation: RenderOperation) -> bool {
        self.operation_support(operation).is_supported()
    }

    pub const fn require_operation(
        &self,
        operation: RenderOperation,
    ) -> Result<RenderOperationSupport, UnsupportedRenderOperation> {
        let support = self.operation_support(operation);
        if support.is_supported() {
            Ok(support)
        } else {
            Err(UnsupportedRenderOperation { operation, support })
        }
    }

    pub fn with_surface_support(mut self, enabled: bool) -> Self {
        self.supports_surface = enabled;
        self
    }

    pub fn with_offscreen_support(mut self, enabled: bool) -> Self {
        self.supports_offscreen = enabled;
        self
    }

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.supports_async_compute = enabled;
        self
    }

    pub fn with_async_copy(mut self, enabled: bool) -> Self {
        self.supports_async_copy = enabled;
        self
    }

    pub fn with_pipeline_cache(mut self, enabled: bool) -> Self {
        self.supports_pipeline_cache = enabled;
        self
    }

    pub fn with_gpu_timestamp(mut self, enabled: bool) -> Self {
        self.supports_gpu_timestamp = enabled;
        self
    }

    pub fn with_subgroup(mut self, enabled: bool) -> Self {
        self.supports_subgroup = enabled;
        self
    }

    pub fn with_pipeline_statistics_query(mut self, enabled: bool) -> Self {
        self.supports_pipeline_statistics_query = enabled;
        self
    }

    pub fn with_storage_buffers(mut self, enabled: bool) -> Self {
        self.supports_storage_buffers = enabled;
        self
    }

    pub fn with_fragment_writable_storage(mut self, enabled: bool) -> Self {
        self.supports_fragment_writable_storage = enabled;
        self
    }

    pub fn with_max_storage_buffers_per_shader_stage(mut self, limit: u32) -> Self {
        self.max_storage_buffers_per_shader_stage = limit;
        self
    }

    pub fn with_max_storage_buffer_binding_size(mut self, limit: u64) -> Self {
        self.max_storage_buffer_binding_size = limit;
        self
    }

    pub fn with_indirect_draw(mut self, enabled: bool) -> Self {
        self.supports_indirect_draw = enabled;
        self
    }

    pub fn with_multi_draw_indirect(mut self, enabled: bool) -> Self {
        self.supports_multi_draw_indirect = enabled;
        self
    }

    pub fn with_multi_draw_indirect_count(mut self, enabled: bool) -> Self {
        self.supports_multi_draw_indirect_count = enabled;
        self
    }

    pub fn with_indirect_first_instance(mut self, enabled: bool) -> Self {
        self.supports_indirect_first_instance = enabled;
        self
    }

    pub fn with_buffer_readback(mut self, enabled: bool) -> Self {
        self.supports_buffer_readback = enabled;
        self
    }

    pub fn with_buffer_binding_array(mut self, enabled: bool) -> Self {
        self.supports_buffer_binding_array = enabled;
        self
    }

    pub fn with_texture_binding_array(mut self, enabled: bool) -> Self {
        self.supports_texture_binding_array = enabled;
        self
    }

    pub fn with_non_uniform_resource_indexing(mut self, enabled: bool) -> Self {
        self.supports_non_uniform_resource_indexing = enabled;
        self
    }

    pub fn with_partially_bound_binding_array(mut self, enabled: bool) -> Self {
        self.supports_partially_bound_binding_array = enabled;
        self
    }

    pub fn with_neural_compute(mut self, enabled: bool) -> Self {
        self.supports_neural_compute = enabled;
        self
    }

    pub fn with_sparse_texture(mut self, enabled: bool) -> Self {
        self.supports_sparse_texture = enabled;
        self
    }

    pub fn with_debug_markers(mut self, enabled: bool) -> Self {
        self.supports_debug_markers = enabled;
        self
    }

    pub fn with_debug_groups(mut self, enabled: bool) -> Self {
        self.supports_debug_groups = enabled;
        self
    }

    pub fn with_graphics_debugger_capture(mut self, enabled: bool) -> Self {
        self.supports_graphics_debugger_capture = enabled;
        self
    }

    pub fn with_acceleration_structures(mut self, caps: AccelerationStructureCaps) -> Self {
        self.acceleration_structures = caps;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDebugInstrumentationStatus {
    pub backend_name: String,
    pub debug_markers_supported: bool,
    pub debug_groups_supported: bool,
    pub graphics_debugger_capture_supported: bool,
    pub active_graphics_debugger_capture: bool,
    pub last_error: Option<String>,
}

impl RenderDebugInstrumentationStatus {
    pub fn from_caps(caps: &RenderBackendCaps) -> Self {
        Self {
            backend_name: caps.backend_name.clone(),
            debug_markers_supported: caps.supports_debug_markers,
            debug_groups_supported: caps.supports_debug_groups,
            graphics_debugger_capture_supported: caps.supports_graphics_debugger_capture,
            active_graphics_debugger_capture: false,
            last_error: None,
        }
    }

    pub fn unavailable(backend_name: impl Into<String>) -> Self {
        Self {
            backend_name: backend_name.into(),
            debug_markers_supported: false,
            debug_groups_supported: false,
            graphics_debugger_capture_supported: false,
            active_graphics_debugger_capture: false,
            last_error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderBackendCaps, RenderOperation, RenderOperationSupport};

    #[test]
    fn render_backend_caps_deserialize_literal_pre_device_diagnostics_payload() {
        let legacy = r#"{
            "backend_name": "wgpu(vulkan)",
            "queue_classes": ["Graphics"],
            "supports_surface": true,
            "supports_offscreen": true,
            "supports_async_compute": false,
            "supports_async_copy": true,
            "supports_pipeline_cache": true,
            "supports_storage_buffers": true,
            "supports_indirect_draw": true,
            "supports_multi_draw_indirect": false,
            "supports_indirect_first_instance": false,
            "supports_buffer_readback": true,
            "supports_buffer_binding_array": false,
            "supports_texture_binding_array": false,
            "supports_non_uniform_resource_indexing": false,
            "supports_partially_bound_binding_array": false,
            "supports_neural_compute": false,
            "supports_sparse_texture": false,
            "supports_debug_markers": true,
            "supports_debug_groups": true,
            "supports_graphics_debugger_capture": false,
            "acceleration_structures": {
                "supported": false,
                "inline_ray_query": false,
                "ray_tracing_pipeline": false,
                "max_instance_count": null
            }
        }"#;

        let decoded: RenderBackendCaps =
            serde_json::from_str(legacy).expect("deserialize literal legacy backend caps");

        assert_eq!(decoded.backend_name, "wgpu(vulkan)");
        assert!(decoded.adapter.is_none());
        assert!(decoded.device_limits.is_none());
        assert!(decoded.supports_storage_buffers);
        assert!(decoded.supports_buffer_readback);
        assert!(!decoded.supports_subgroup);
        assert!(!decoded.supports_pipeline_statistics_query);
        assert_eq!(
            decoded.operation_support(RenderOperation::DirectDraw),
            RenderOperationSupport::Unsupported
        );
    }
}
