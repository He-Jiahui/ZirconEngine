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
            Self::ScreenSpaceAntiAlias => capabilities.supports_fxaa || capabilities.supports_smaa,
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
