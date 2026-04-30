use crate::core::framework::render::{RenderCapabilityKind, RenderCapabilitySummary};

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
        self.capability_kind().is_satisfied_by(capabilities)
    }

    pub const fn capability_kind(self) -> RenderCapabilityKind {
        match self {
            Self::VirtualGeometry => RenderCapabilityKind::VirtualGeometry,
            Self::HybridGlobalIllumination => RenderCapabilityKind::HybridGlobalIllumination,
            Self::AccelerationStructures => RenderCapabilityKind::AccelerationStructures,
            Self::InlineRayQuery => RenderCapabilityKind::InlineRayQuery,
            Self::RayTracingPipeline => RenderCapabilityKind::RayTracingPipeline,
            Self::AsyncCompute => RenderCapabilityKind::AsyncCompute,
            Self::AsyncCopy => RenderCapabilityKind::AsyncCopy,
        }
    }

    pub const fn label(self) -> &'static str {
        self.capability_kind().label()
    }
}
