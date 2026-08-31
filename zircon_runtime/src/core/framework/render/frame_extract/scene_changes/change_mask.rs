use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderComponentChangeMask(u8);

impl RenderComponentChangeMask {
    pub const MESH_RENDERER: Self = Self(1 << 0);
    pub const WORLD_TRANSFORM: Self = Self(1 << 1);
    pub const ACTIVE_IN_HIERARCHY: Self = Self(1 << 2);
    pub const RENDER_LAYER: Self = Self(1 << 3);
    pub const MOBILITY: Self = Self(1 << 4);
    pub const ALL: Self = Self(
        Self::MESH_RENDERER.0
            | Self::WORLD_TRANSFORM.0
            | Self::ACTIVE_IN_HIERARCHY.0
            | Self::RENDER_LAYER.0
            | Self::MOBILITY.0,
    );

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for RenderComponentChangeMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RenderComponentChangeMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
