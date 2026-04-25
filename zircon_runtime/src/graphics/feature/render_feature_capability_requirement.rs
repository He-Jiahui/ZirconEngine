use crate::core::framework::render::RenderCapabilitySummary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderFeatureCapabilityRequirement {
    VirtualGeometry,
    HybridGlobalIllumination,
    AccelerationStructures,
    InlineRayQuery,
    RayTracingPipeline,
    AsyncCompute,
    AsyncCopy,
}

impl RenderFeatureCapabilityRequirement {
    pub fn is_satisfied_by(self, capabilities: &RenderCapabilitySummary) -> bool {
        match self {
            Self::VirtualGeometry => capabilities.virtual_geometry_supported,
            Self::HybridGlobalIllumination => capabilities.hybrid_global_illumination_supported,
            Self::AccelerationStructures => capabilities.acceleration_structures_supported,
            Self::InlineRayQuery => capabilities.inline_ray_query,
            Self::RayTracingPipeline => capabilities.ray_tracing_pipeline,
            Self::AsyncCompute => capabilities.supports_async_compute,
            Self::AsyncCopy => capabilities.supports_async_copy,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
            Self::AccelerationStructures => "acceleration_structures",
            Self::InlineRayQuery => "inline_ray_query",
            Self::RayTracingPipeline => "ray_tracing_pipeline",
            Self::AsyncCompute => "async_compute",
            Self::AsyncCopy => "async_copy",
        }
    }
}
