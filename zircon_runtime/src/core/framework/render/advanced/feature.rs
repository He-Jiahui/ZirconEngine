use super::super::{RenderCapabilityKind, RenderProductFeature};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdvancedRenderFeature {
    VirtualGeometry,
    HybridGlobalIllumination,
}

impl AdvancedRenderFeature {
    pub const ALL: [Self; 2] = [Self::VirtualGeometry, Self::HybridGlobalIllumination];

    pub const fn label(self) -> &'static str {
        match self {
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
        }
    }

    pub const fn product_feature(self) -> RenderProductFeature {
        match self {
            Self::VirtualGeometry => RenderProductFeature::VirtualGeometry,
            Self::HybridGlobalIllumination => RenderProductFeature::HybridGlobalIllumination,
        }
    }

    pub const fn required_capability(self) -> RenderCapabilityKind {
        match self {
            Self::VirtualGeometry => RenderCapabilityKind::VirtualGeometry,
            Self::HybridGlobalIllumination => RenderCapabilityKind::HybridGlobalIllumination,
        }
    }

    pub const fn required_capabilities(self) -> &'static [RenderCapabilityKind] {
        match self {
            Self::VirtualGeometry => &[
                RenderCapabilityKind::VirtualGeometry,
                RenderCapabilityKind::StorageBuffers,
                RenderCapabilityKind::IndirectDraw,
                RenderCapabilityKind::BufferReadback,
            ],
            Self::HybridGlobalIllumination => &[
                RenderCapabilityKind::HybridGlobalIllumination,
                RenderCapabilityKind::StorageBuffers,
                RenderCapabilityKind::BufferReadback,
            ],
        }
    }
}
