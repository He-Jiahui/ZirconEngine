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

    pub fn with_acceleration_structures(mut self, caps: AccelerationStructureCaps) -> Self {
        self.acceleration_structures = caps;
        self
    }
}
