use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderQueueClass {
    Graphics,
    Compute,
    Copy,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderBackendCaps {
    pub backend_name: String,
    pub queue_classes: Vec<RenderQueueClass>,
    pub supports_surface: bool,
    pub supports_offscreen: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
    pub supports_pipeline_cache: bool,
    pub supports_storage_buffers: bool,
    #[serde(default)]
    pub supports_fragment_writable_storage: bool,
    #[serde(default)]
    pub max_storage_buffers_per_shader_stage: u32,
    #[serde(default)]
    pub max_storage_buffer_binding_size: u64,
    pub supports_indirect_draw: bool,
    pub supports_multi_draw_indirect: bool,
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
}

impl RenderBackendCaps {
    pub fn new(backend_name: impl Into<String>) -> Self {
        Self {
            backend_name: backend_name.into(),
            queue_classes: Vec::new(),
            supports_surface: false,
            supports_offscreen: true,
            supports_async_compute: false,
            supports_async_copy: false,
            supports_pipeline_cache: false,
            supports_storage_buffers: false,
            supports_fragment_writable_storage: false,
            max_storage_buffers_per_shader_stage: 0,
            max_storage_buffer_binding_size: 0,
            supports_indirect_draw: false,
            supports_multi_draw_indirect: false,
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
        }
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
