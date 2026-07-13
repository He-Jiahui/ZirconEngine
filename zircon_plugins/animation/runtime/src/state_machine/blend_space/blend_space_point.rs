use zircon_runtime::core::math::{Real, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendSpacePoint1D {
    pub(super) position: Real,
    pub(super) sample: u32,
}

impl BlendSpacePoint1D {
    pub const fn new(position: Real, sample: u32) -> Self {
        Self { position, sample }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendSpacePoint2D {
    pub(super) position: Vec2,
    pub(super) sample: u32,
}

impl BlendSpacePoint2D {
    pub const fn new(position: Vec2, sample: u32) -> Self {
        Self { position, sample }
    }
}
