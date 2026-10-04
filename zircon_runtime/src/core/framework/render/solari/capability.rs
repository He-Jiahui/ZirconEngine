use super::super::RenderCapabilityKind;

/// Solari 实验路径的硬件前提集合；报告逐项记录缺失能力而非只检查总开关。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SolariCapabilityRequirement {
    InlineRayQuery,
    AccelerationStructures,
    BufferBindingArray,
    TextureBindingArray,
    NonUniformResourceIndexing,
    PartiallyBoundBindingArray,
}

impl SolariCapabilityRequirement {
    pub const ALL: [Self; 6] = [
        Self::InlineRayQuery,
        Self::AccelerationStructures,
        Self::BufferBindingArray,
        Self::TextureBindingArray,
        Self::NonUniformResourceIndexing,
        Self::PartiallyBoundBindingArray,
    ];

    pub const fn capability_kind(self) -> RenderCapabilityKind {
        match self {
            Self::InlineRayQuery => RenderCapabilityKind::InlineRayQuery,
            Self::AccelerationStructures => RenderCapabilityKind::AccelerationStructures,
            Self::BufferBindingArray => RenderCapabilityKind::BufferBindingArray,
            Self::TextureBindingArray => RenderCapabilityKind::TextureBindingArray,
            Self::NonUniformResourceIndexing => RenderCapabilityKind::NonUniformResourceIndexing,
            Self::PartiallyBoundBindingArray => RenderCapabilityKind::PartiallyBoundBindingArray,
        }
    }

    pub const fn label(self) -> &'static str {
        self.capability_kind().label()
    }
}
