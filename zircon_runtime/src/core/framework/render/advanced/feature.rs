use super::super::{RenderCapabilityKind, RenderProductFeature};

/// 产品 profile 用稳定特性键生成 provider 资格报告；渲染管线还需独立启用对应 pass。
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

    /// 运行高级 provider 需要完整后端能力，不能只检查同名产品能力位。
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
