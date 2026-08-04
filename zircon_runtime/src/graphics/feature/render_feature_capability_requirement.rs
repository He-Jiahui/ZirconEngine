use crate::core::framework::render::{RenderCapabilityKind, RenderCapabilitySummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderFeatureCapabilityRequirement {
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
    SubgroupOps,
    PipelineStatisticsQuery,
}

impl RenderFeatureCapabilityRequirement {
    pub fn is_satisfied_by(self, capabilities: &RenderCapabilitySummary) -> bool {
        self.capability_kind().is_satisfied_by(capabilities)
    }

    pub const fn capability_kind(self) -> RenderCapabilityKind {
        match self {
            Self::VirtualGeometry => RenderCapabilityKind::VirtualGeometry,
            Self::HybridGlobalIllumination => RenderCapabilityKind::HybridGlobalIllumination,
            Self::AccelerationStructures => RenderCapabilityKind::AccelerationStructures,
            Self::InlineRayQuery => RenderCapabilityKind::InlineRayQuery,
            Self::RayTracingPipeline => RenderCapabilityKind::RayTracingPipeline,
            Self::BufferBindingArray => RenderCapabilityKind::BufferBindingArray,
            Self::TextureBindingArray => RenderCapabilityKind::TextureBindingArray,
            Self::NonUniformResourceIndexing => RenderCapabilityKind::NonUniformResourceIndexing,
            Self::PartiallyBoundBindingArray => RenderCapabilityKind::PartiallyBoundBindingArray,
            Self::ScreenSpaceAntiAlias => RenderCapabilityKind::ScreenSpaceAntiAlias,
            Self::StorageBuffers => RenderCapabilityKind::StorageBuffers,
            Self::IndirectDraw => RenderCapabilityKind::IndirectDraw,
            Self::BufferReadback => RenderCapabilityKind::BufferReadback,
            Self::AsyncCompute => RenderCapabilityKind::AsyncCompute,
            Self::AsyncCopy => RenderCapabilityKind::AsyncCopy,
            Self::NeuralCompute => RenderCapabilityKind::NeuralCompute,
            Self::SparseTexture => RenderCapabilityKind::SparseTexture,
            Self::SubgroupOps => RenderCapabilityKind::SubgroupOps,
            Self::PipelineStatisticsQuery => RenderCapabilityKind::PipelineStatisticsQuery,
        }
    }

    pub const fn from_capability_kind(kind: RenderCapabilityKind) -> Self {
        match kind {
            RenderCapabilityKind::VirtualGeometry => Self::VirtualGeometry,
            RenderCapabilityKind::HybridGlobalIllumination => Self::HybridGlobalIllumination,
            RenderCapabilityKind::AccelerationStructures => Self::AccelerationStructures,
            RenderCapabilityKind::InlineRayQuery => Self::InlineRayQuery,
            RenderCapabilityKind::RayTracingPipeline => Self::RayTracingPipeline,
            RenderCapabilityKind::BufferBindingArray => Self::BufferBindingArray,
            RenderCapabilityKind::TextureBindingArray => Self::TextureBindingArray,
            RenderCapabilityKind::NonUniformResourceIndexing => Self::NonUniformResourceIndexing,
            RenderCapabilityKind::PartiallyBoundBindingArray => Self::PartiallyBoundBindingArray,
            RenderCapabilityKind::ScreenSpaceAntiAlias => Self::ScreenSpaceAntiAlias,
            RenderCapabilityKind::StorageBuffers => Self::StorageBuffers,
            RenderCapabilityKind::IndirectDraw => Self::IndirectDraw,
            RenderCapabilityKind::BufferReadback => Self::BufferReadback,
            RenderCapabilityKind::AsyncCompute => Self::AsyncCompute,
            RenderCapabilityKind::AsyncCopy => Self::AsyncCopy,
            RenderCapabilityKind::NeuralCompute => Self::NeuralCompute,
            RenderCapabilityKind::SparseTexture => Self::SparseTexture,
            RenderCapabilityKind::SubgroupOps => Self::SubgroupOps,
            RenderCapabilityKind::PipelineStatisticsQuery => Self::PipelineStatisticsQuery,
        }
    }

    pub const fn label(self) -> &'static str {
        self.capability_kind().label()
    }
}

#[cfg(test)]
mod tests {
    use super::RenderFeatureCapabilityRequirement;
    use crate::core::framework::render::RenderCapabilityKind;

    #[test]
    fn feature_requirements_round_trip_every_render_capability() {
        for capability in RenderCapabilityKind::ALL {
            let requirement = RenderFeatureCapabilityRequirement::from_capability_kind(capability);

            assert_eq!(requirement.capability_kind(), capability);
        }
    }
}
