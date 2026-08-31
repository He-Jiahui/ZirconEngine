use std::ops::{BitOr, BitOrAssign};

/// Backend-neutral shader visibility bits for graph resource use intent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderGraphShaderStages(u8);

impl RenderGraphShaderStages {
    pub const NONE: Self = Self(0);
    pub const VERTEX: Self = Self(1 << 0);
    pub const FRAGMENT: Self = Self(1 << 1);
    pub const COMPUTE: Self = Self(1 << 2);
    pub const ALL_GRAPHICS: Self = Self(Self::VERTEX.0 | Self::FRAGMENT.0);
    pub const ALL: Self = Self(Self::ALL_GRAPHICS.0 | Self::COMPUTE.0);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for RenderGraphShaderStages {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RenderGraphShaderStages {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
